use crate::{
    api::Api,
    proxy,
    settings::{Config, Settings},
};
use garlic_state::StateError;
use jni_sys::{jclass, jint, jstring, JNIEnv};
use serde_json::{json, Value};
use std::{
    ffi::c_char,
    net::SocketAddr,
    path::Path,
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

type Reply = Result<Value, StateError>;

#[no_mangle]
pub extern "C" fn garlic_core_abi() -> jint {
    1
}
#[no_mangle]
pub extern "C" fn garlic_core_version() -> *const c_char {
    c"0.9.2".as_ptr()
}

unsafe fn from_java(env: *mut JNIEnv, s: jstring) -> Result<String, StateError> {
    if s.is_null() {
        return Err("null Java string".into());
    }
    let j = &(**env).v1_2;
    let len = (j.GetStringLength)(env, s);
    if !(0..=1048576).contains(&len) {
        return Err("Java string too large".into());
    }
    let chars = (j.GetStringChars)(env, s, ptr::null_mut());
    if chars.is_null() {
        return Err("could not read Java string".into());
    }
    let result = String::from_utf16(std::slice::from_raw_parts(chars, len as usize));
    (j.ReleaseStringChars)(env, s, chars);
    Ok(result?)
}
unsafe fn java_text(env: *mut JNIEnv, s: &str) -> jstring {
    let units: Vec<u16> = s.encode_utf16().collect();
    ((**env).v1_2.NewString)(env, units.as_ptr(), units.len() as jint)
}
unsafe fn result(env: *mut JNIEnv, reply: Reply) -> jstring {
    let v = match reply {
        Ok(mut v) => {
            if let Some(o) = v.as_object_mut() {
                o.entry("ok").or_insert(Value::Bool(true));
            }
            v
        }
        Err(e) => json!({"ok":false,"error":e.to_string()}),
    };
    java_text(env, &v.to_string())
}
macro_rules! entry {
    ($name:ident,($($arg:ident),*),$body:block)=>{
        #[no_mangle]
        pub unsafe extern "system" fn $name(env:*mut JNIEnv,_class:jclass,$($arg:jstring),*)->jstring{
            if env.is_null(){return ptr::null_mut();}
            result(env,(||->Reply { $(let $arg=from_java(env,$arg)?;)* $body })())
        }
    };
}
fn profile(dir: &str) -> Result<Settings, StateError> {
    let path = Path::new(dir);
    if !path.is_absolute() {
        return Err("expected application files directory".into());
    }
    std::fs::create_dir_all(path)?;
    Ok(Settings {
        config: path.join("client.json"),
    })
}
fn urls(value: &str) -> Result<Vec<String>, StateError> {
    let mut out = Vec::new();
    for raw in value.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let u = crate::settings::origin(raw)?;
        if !out.contains(&u) {
            out.push(u);
        }
    }
    Ok(out)
}
fn android_api(dir: &str, origins: &str) -> Result<(Settings, Config, Api), StateError> {
    let settings = profile(dir)?;
    let mut cfg = settings.load()?;
    cfg.control_urls = urls(origins)?;
    cfg.storage_urls.clear();
    let api = Api::new(&cfg)?;
    Ok((settings, cfg, api))
}
fn admin(dir: &str, origins: &str, operation: &str, body: Option<&str>) -> Reply {
    let (settings, cfg, api) = android_api(dir, origins)?;
    let data = body.map(serde_json::from_str).transpose()?;
    api.admin(&settings, &cfg, operation, data.as_ref())
}
entry!(
    Java_org_garlic_client_GarlicBridge_connectNetwork,
    (dir, url, key),
    {
        let settings = profile(&dir)?;
        Api::new(&settings.load()?)?.connect(&settings, &url, &key)
    }
);
entry!(
    Java_org_garlic_client_GarlicBridge_saveKey,
    (dir, origins, key),
    { profile(&dir)?.save_key(&key, urls(&origins)?) }
);
entry!(
    Java_org_garlic_client_GarlicBridge_loginMulti,
    (dir, origins, name, password),
    {
        let (settings, cfg, api) = android_api(&dir, &origins)?;
        let mut accepted = vec![];
        let mut last: StateError = "no endpoints available".into();
        for url in cfg.control_urls {
            match api.login(&settings, &url, &name, &password) {
                Ok(_) => accepted.push(url),
                Err(e) => last = e,
            }
        }
        if accepted.is_empty() {
            return Err(last);
        }
        Ok(json!({"ok":true,"origins":accepted}))
    }
);
entry!(
    Java_org_garlic_client_GarlicBridge_account,
    (dir, origins),
    { admin(&dir, &origins, "account", None) }
);
entry!(
    Java_org_garlic_client_GarlicBridge_adminListUsers,
    (dir, origins),
    { admin(&dir, &origins, "list", None) }
);
entry!(
    Java_org_garlic_client_GarlicBridge_adminManageUser,
    (dir, origins, body),
    { admin(&dir, &origins, "manage", Some(&body)) }
);
entry!(
    Java_org_garlic_client_GarlicBridge_adminNetwork,
    (dir, origins, body),
    {
        if body.trim().is_empty() {
            admin(&dir, &origins, "network", None)
        } else {
            admin(&dir, &origins, "network-set", Some(&body))
        }
    }
);
entry!(
    Java_org_garlic_client_GarlicBridge_takeover,
    (dir, origins),
    {
        let (settings, cfg, api) = android_api(&dir, &origins)?;
        api.takeover(&settings, &cfg)
    }
);
#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_GarlicBridge_generatePac(
    env: *mut JNIEnv,
    _: jclass,
    s: jstring,
) -> jstring {
    if env.is_null() {
        return ptr::null_mut();
    }
    match from_java(env, s).and_then(|v| Ok(v.parse::<SocketAddr>()?)) {
        Ok(addr) => java_text(env, &garlic_client_logic::generate_pac(&addr)),
        Err(e) => result(env, Err(e)),
    }
}
struct Running {
    address: SocketAddr,
    #[cfg(target_os = "android")]
    tunnel: Option<crate::tunnel::Tunnel>,
    api: Api,
    stop: Arc<AtomicBool>,
    thread: thread::JoinHandle<Result<(), StateError>>,
}
static RUNNING: Mutex<Option<Running>> = Mutex::new(None);
entry!(
    Java_org_garlic_client_GarlicBridge_startProxy,
    (dir, mode, direct, control, bind),
    {
        let mut guard = RUNNING.lock().map_err(|_| "proxy state unavailable")?;
        if guard.as_ref().is_some_and(|v| !v.thread.is_finished()) {
            return Err("proxy is already running".into());
        }
        let settings = profile(&dir)?;
        let mut cfg = settings.edit(|c| {
            c.set("mode", &mode)?;
            c.storage_urls = urls(&direct)?;
            c.control_urls = urls(&control)?;
            Ok(())
        })?;
        proxy::apply_connections(&settings, &mut cfg)?;
        let api = Api::new(&cfg)?;
        let api_thread = api.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let addr = bind.parse()?;
        let (tx, rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("garlic-android-proxy".into())
            .spawn(move || {
                proxy::serve(&settings, &cfg, api_thread, addr, signal, Some(tx), None)
            })?;
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(address) => {
                *guard = Some(Running {
                    address,
                    #[cfg(target_os = "android")]
                    tunnel: None,
                    api,
                    stop,
                    thread,
                });
                Ok(json!({"ok":true,"proxy_bind":address.to_string()}))
            }
            Err(e) => {
                stop.store(true, Ordering::Release);
                if thread.is_finished() {
                    thread
                        .join()
                        .map_err(|_| "proxy thread ended unexpectedly")??;
                }
                Err(e.into())
            }
        }
    }
);
entry!(Java_org_garlic_client_GarlicBridge_status, (), {
    let state = RUNNING
        .lock()
        .map_err(|_| "proxy state unavailable")?
        .as_ref()
        .filter(|running| !running.thread.is_finished())
        .map(|v| {
            #[cfg(target_os = "android")]
            let tunnel = v.tunnel.as_ref().is_some_and(|t| t.active());
            #[cfg(not(target_os = "android"))]
            let tunnel = false;
            (v.api.clone(), v.address, tunnel)
        });
    if let Some((api, addr, tunnel)) = state {
        let mut status = api.proxy_status(addr)?;
        status["proxy"] = json!(true);
        status["tunnel"] = json!(tunnel);
        Ok(status)
    } else {
        Ok(json!({"network_ready":false,"active_streams":0,"proxy":false,"tunnel":false}))
    }
});
entry!(Java_org_garlic_client_GarlicBridge_stopProxy, (), {
    let mut guard = RUNNING.lock().map_err(|_| "proxy state unavailable")?;
    if let Some(r) = guard.take() {
        r.stop.store(true, Ordering::Release);
        #[cfg(target_os = "android")]
        if let Some(tun) = r.tunnel {
            tun.shutdown()?;
        }
        r.thread.join().map_err(|_| "proxy thread terminated")??;
    }
    Ok(json!({"ok":true}))
});
#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_GarlicBridge_startTun(
    env: *mut JNIEnv,
    _: jclass,
    fd: jint,
) -> jstring {
    if env.is_null() {
        return ptr::null_mut();
    }
    #[cfg(target_os = "android")]
    let started = (|| -> Reply {
        let mut guard = RUNNING.lock().map_err(|_| "proxy state unavailable")?;
        let r = guard
            .as_mut()
            .ok_or("start the proxy before attaching a TUN interface")?;
        if r.tunnel.as_ref().is_some_and(|t| t.active()) {
            return Err("TUN already running".into());
        }
        r.tunnel = Some(crate::tunnel::android(fd, r.address)?);
        Ok(json!({"ok":true}))
    })();
    #[cfg(not(target_os = "android"))]
    let started = {
        let _ = fd;
        Err("Android descriptor required".into())
    };
    result(env, started)
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_ModuleCanvas_open(env:*mut JNIEnv,_:jclass,code:jstring)->jint {
    from_java(env,code).ok().is_some_and(|s|crate::plugin_view::open(&s)) as jint
}
#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_ModuleCanvas_tick(env:*mut JNIEnv,_:jclass,x:jint,y:jint,d:jint,ms:jint)->jni_sys::jintArray {
    let Some(frame)=crate::plugin_view::tick(x,y,d,ms) else{return ptr::null_mut()};
    let j=&(**env).v1_2;
    let arr=(j.NewIntArray)(env,frame.len() as jint);
    if !arr.is_null(){(j.SetIntArrayRegion)(env,arr,0,frame.len() as jint,frame.as_ptr());}
    arr
}
#[no_mangle]
pub extern "system" fn Java_org_garlic_client_ModuleCanvas_close(_: *mut JNIEnv,_:jclass){crate::plugin_view::garlic_view_close();}
