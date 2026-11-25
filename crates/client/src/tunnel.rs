use garlic_state::StateError;
use std::{net::SocketAddr, thread};
use tun2proxy::{ArgDns, Args, CancellationToken};

fn options(proxy: SocketAddr) -> Result<Args, StateError> {
    if !proxy.ip().is_loopback() {
        return Err("expected local Garlic proxy".into());
    }
    let mut options = Args::default();
    options
        .proxy(format!("socks5://{proxy}").parse()?)
        .dns(ArgDns::Virtual);
    options.dns_addr = "1.1.1.1".parse()?;
    options.mtu = 1500;
    Ok(options)
}

pub struct Tunnel {
    cancel: CancellationToken,
    thread: Option<thread::JoinHandle<Result<(), StateError>>>,
}
impl Tunnel {
    fn start(
        f: impl FnOnce(CancellationToken) -> Result<(), StateError> + Send + 'static,
    ) -> Result<Self, StateError> {
        let cancel = CancellationToken::new();
        let runner = cancel.clone();
        let thread = thread::Builder::new()
            .name("garlic-tun".into())
            .spawn(move || f(runner))?;
        Ok(Self {
            cancel,
            thread: Some(thread),
        })
    }
    pub fn active(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }
    pub fn shutdown(mut self) -> Result<(), StateError> {
        self.cancel.cancel();
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| "TUN task terminated")??;
        }
        Ok(())
    }
}
impl Drop for Tunnel {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
fn runtime() -> Result<tokio::runtime::Runtime, StateError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?)
}

#[cfg(all(test, windows))]
mod packet_tests;

#[cfg(windows)]
mod windows_network;

#[cfg(windows)]
pub fn run_vpn(
    settings: crate::settings::Settings,
    mut cfg: crate::settings::Config,
) -> Result<(), StateError> {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    };
    crate::proxy::apply_connections(&settings, &mut cfg)?;
    let (api, bypass) = crate::api::Api::for_windows_vpn(&mut cfg)?;
    let stop = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let (tx, rx) = mpsc::sync_channel(1);
    let proxy_stop = stop.clone();
    let proxy_ready = ready.clone();
    let proxy_cfg = cfg.clone();
    let proxy_task = thread::spawn(move || {
        crate::proxy::serve(
            &settings,
            &proxy_cfg,
            api,
            "127.0.0.1:1080".parse()?,
            proxy_stop,
            Some(tx),
            Some(proxy_ready),
        )
    });
    let result = (|| -> Result<(), StateError> {
        let proxy = rx.recv_timeout(std::time::Duration::from_secs(10))?;
        let tunnel = windows(proxy, bypass, ready)?;
        while tunnel.active() && !proxy_task.is_finished() && !stop.load(Ordering::Acquire) {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        tunnel.shutdown()
    })();
    stop.store(true, Ordering::Release);
    proxy_task.join().map_err(|_| "proxy task terminated")??;
    result
}

#[cfg(target_os = "android")]
pub fn android(fd: i32, proxy: SocketAddr) -> Result<Tunnel, StateError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    unsafe extern "C" {
        fn dup(fd: i32) -> i32;
    }
    let duplicate = unsafe { dup(fd) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let owned = unsafe { OwnedFd::from_raw_fd(duplicate) };
    let mut opts = options(proxy)?;
    opts.setup(false);
    let rt = runtime()?;
    let mut cfg = tun::Configuration::default();
    cfg.raw_fd(owned.as_raw_fd())
        .close_fd_on_drop(false)
        .mtu(1500);
    let device = {
        let _entered = rt.enter();
        tun::create_as_async(&cfg)?
    };
    Tunnel::start(move |cancel| {
        // Java retains its ParcelFileDescriptor. Our duplicate lives until the runner has stopped.
        let _fd = owned;
        rt.block_on(tun2proxy::run(device, 1500, opts, cancel))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(windows)]
fn windows(
    proxy: SocketAddr,
    bypass: std::collections::BTreeSet<std::net::Ipv4Addr>,
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<Tunnel, StateError> {
    let mut opts = options(proxy)?;
    opts.setup(false);
    let rt = runtime()?;
    Tunnel::start(move |token| {
        rt.block_on(async move {
            use std::sync::atomic::Ordering;
            let mut network = windows_network::Guard::lock()?;
            let mut cfg = tun::Configuration::default();
            cfg.tun_name("Garlic")
                .address(tproxy_config::TUN_IPV4)
                .netmask(tproxy_config::TUN_NETMASK)
                .mtu(1500)
                .up();
            let mut retries = 0;
            let dev = loop {
                match tun::create_as_async(&cfg) {
                    Ok(dev) => break dev,
                    Err(error) if retries < 2 && wintun_class_pending(&error) => {
                        retries += 1;
                        eprintln!("Waiting for Windows to finish configuring Wintun");
                        tokio::select! {
                            _ = token.cancelled() => return Ok(()),
                            _ = tokio::time::sleep(std::time::Duration::from_millis(1500)) => {}
                        }
                    }
                    Err(error) => return Err(error.into()),
                }
            };
            if token.is_cancelled() {
                return Ok(());
            }
            network.enable(bypass)?;
            network.activate()?;
            ready.store(true, Ordering::Release);
            let result = tun2proxy::run(dev, 1500, opts, token).await;
            ready.store(false, Ordering::Release);
            network.disable()?;
            result.map_err(|e| e.to_string())?;
            Ok(())
        })
    })
}

#[cfg(windows)]
fn wintun_class_pending(error: &tun::Error) -> bool {
    let tun::Error::WintunError(error) = error else {
        return false;
    };
    // The binding preserves the driver diagnostic as text. Match the PnP code,
    // since other reasons for an unavailable adapter should fail immediately.
    let detail = error.to_string().to_ascii_lowercase();
    detail.starts_with("wintuncreateadapter failed") && detail.contains("problem code: 0x38,")
}

#[cfg(all(test, windows))]
#[test]
fn recognizes_only_the_pending_wintun_class_configuration() {
    let pending = tun::Error::WintunError(
        "WintunCreateAdapter failed \"Failed to setup adapter (problem code: 0x38, ntstatus: 0x0)\""
            .into(),
    );
    assert!(wintun_class_pending(&pending));
    let unrelated = tun::Error::WintunError(
        "WintunCreateAdapter failed \"Failed to setup adapter (problem code: 0x384, ntstatus: 0x0)\""
            .into(),
    );
    assert!(!wintun_class_pending(&unrelated));
    assert!(!wintun_class_pending(&tun::Error::InvalidConfig));
}
