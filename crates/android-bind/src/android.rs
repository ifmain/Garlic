use core::ffi::{c_char, c_int, c_void, CStr};
use core::{mem, ptr};
use jni_sys::{jboolean, jclass, jstring, JNIEnv, JNINativeMethod};

#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

macro_rules! method {
    ($name:literal, $args:literal) => {
        (
            concat!($name, "\0"),
            concat!("(", $args, ")Ljava/lang/String;\0"),
            concat!("Java_org_garlic_client_GarlicBridge_", $name, "\0"),
        )
    };
}

const METHODS: [(&str, &str, &str); 13] = [
    method!("generatePac", "Ljava/lang/String;"),
    method!("saveKey", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("loginMulti", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("connectNetwork", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("adminNetwork", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("account", "Ljava/lang/String;Ljava/lang/String;"),
    method!("adminListUsers", "Ljava/lang/String;Ljava/lang/String;"),
    method!("adminManageUser", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("startProxy", "Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;"),
    method!("takeover", "Ljava/lang/String;Ljava/lang/String;"),
    method!("status", ""), method!("startTun", "I"), method!("stopProxy", ""),
];

unsafe fn register(env: *mut JNIEnv, handle: *mut c_void, requested: jstring) -> bool {
    let jni = &(**env).v1_2;
    let abi = dlsym(handle, c"garlic_core_abi".as_ptr());
    let version = dlsym(handle, c"garlic_core_version".as_ptr());
    if abi.is_null()
        || mem::transmute::<_, unsafe extern "C" fn() -> c_int>(abi)() != 1
        || version.is_null()
    {
        return false;
    }
    let reported = mem::transmute::<_, unsafe extern "C" fn() -> *const c_char>(version)();
    if reported.is_null() {
        return false;
    }
    let required = (jni.GetStringUTFChars)(env, requested, ptr::null_mut());
    if required.is_null() {
        return false;
    }
    let matches = super::version_matches(
        CStr::from_ptr(reported).to_bytes(),
        CStr::from_ptr(required).to_bytes(),
    );
    (jni.ReleaseStringUTFChars)(env, requested, required);
    if !matches {
        return false;
    }
    let class = (jni.FindClass)(env, c"org/garlic/client/GarlicBridge".as_ptr());
    if class.is_null() {
        return false;
    }
    let mut methods = METHODS.map(|(name, signature, _)| JNINativeMethod {
        name: name.as_ptr() as *mut c_char,
        signature: signature.as_ptr() as *mut c_char,
        fnPtr: ptr::null_mut(),
    });
    for (entry, (_, _, symbol)) in methods.iter_mut().zip(METHODS) {
        entry.fnPtr = dlsym(handle, symbol.as_ptr().cast());
        if entry.fnPtr.is_null()
            || (jni.GetStaticMethodID)(env, class, entry.name, entry.signature).is_null()
        {
            return false;
        }
    }
    (jni.RegisterNatives)(env, class, methods.as_ptr(), methods.len() as i32) == 0
}

/// The caller supplies the JNI environment and Java strings for the current native frame.
#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_CoreLoader_bind(
    env: *mut JNIEnv,
    _: jclass,
    path: jstring,
    requested: jstring,
) -> jboolean {
    let jni = &(**env).v1_2;
    let filename = (jni.GetStringUTFChars)(env, path, ptr::null_mut());
    if filename.is_null() {
        return false;
    }
    let handle = dlopen(filename, 2);
    (jni.ReleaseStringUTFChars)(env, path, filename);
    if handle.is_null() {
        return false;
    }
    if register(env, handle, requested) {
        return true;
    }
    if (jni.ExceptionCheck)(env) {
        (jni.ExceptionClear)(env);
    }
    dlclose(handle);
    false
}
