use crate::{
    accounts::AccountsService,
    devices::DeviceStore,
    files,
    http::{self, App},
};
use garlic_relay_data::control::Authority;
use garlic_relay_session::SessionError;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    ServerConfig,
};
use std::{env, fs, path::PathBuf, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

fn path(name: &str) -> Result<PathBuf, SessionError> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing service setting {name}").into())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Coordinator,
    Storage,
}
pub async fn serve(role: Role) -> Result<(), SessionError> {
    let manifest = env::var_os("GARLIC_PROTOCOL_MODULES").map(PathBuf::from);
    let modules = crate::exchanges::modules::Registry::load(manifest.as_deref())?;
    let account_file = path("GARLIC_AUTH_FILE")?;
    let keys = path("GARLIC_INFRA_KEYS_FILE")?;
    let state = if role == Role::Storage {
        path("GARLIC_STORAGE_STATE_FILE")
            .or_else(|_| path("GARLIC_STATE_FILE"))
            .unwrap_or_else(|_| account_file.with_extension("storage.json"))
    } else {
        path("GARLIC_STATE_FILE")?
    };
    let bind = env::var(if role == Role::Storage {
        "GARLIC_STORAGE_BIND"
    } else {
        "GARLIC_DNS_BIND"
    })?;
    let listener = TcpListener::bind(bind).await?;
    let trust_loopback_proxy =
        env::var("GARLIC_TRUST_LOOPBACK_PROXY_PROTOCOL").as_deref() == Ok("1");
    if role == Role::Storage && !account_file.exists() {
        crate::accounts::operator::initialize(&account_file)?;
    }
    let owner_file = state.with_extension("authority.json");
    let owner_guard = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(state.with_extension("authority.lock"))?;
    owner_guard
        .try_lock()
        .map_err(|_| "another coordinator already owns this state")?;
    let owner: Authority = files::edit_or_create(
        &owner_file,
        || {
            Ok(Authority {
                owner_id: Uuid::new_v4(),
                epoch: 0,
            })
        },
        |owner| {
            if owner.owner_id.is_nil() {
                return Err("invalid coordinator authority".into());
            }
            owner.epoch = owner
                .epoch
                .checked_add(1)
                .ok_or("coordinator epoch exhausted")?;
            Ok(owner.clone())
        },
    )?;
    let certs = CertificateDer::pem_slice_iter(&fs::read(path("GARLIC_TLS_CERT")?)?)
        .collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(path("GARLIC_TLS_KEY")?)?;
    let mut tls =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
    tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let acceptor = TlsAcceptor::from(Arc::new(tls));
    let devices = DeviceStore::for_accounts(&account_file);
    let public = env::var("GARLIC_PUBLIC_URL").or_else(|_| env::var("GARLIC_DNS_URL"))?;
    let account_source = env::var("GARLIC_ACCOUNT_AUTHORITY")
        .ok()
        .filter(|u| !u.is_empty() && u.trim_end_matches('/') != public.trim_end_matches('/'));
    let authority = account_source
        .as_deref()
        .map(|origin| {
            crate::upstream::AccountAuthority::new(origin, &fs::read(path("GARLIC_CA_CERT")?)?)
        })
        .transpose()?;
    let jobs = crate::queue::store::Jobs::new(&state)?;
    crate::queue::transfers::recover(&jobs, &state)?;
    let app = Arc::new(App {
        modules,
        update_dir: env::var_os("GARLIC_NETWORK_FILE")
            .map(PathBuf::from)
            .and_then(|path| path.parent().map(|dir| dir.join("core-updates"))),
        service_name: if role == Role::Storage {
            env::var("GARLIC_STORAGE_NAME").unwrap_or_else(|_| "storage-a".into())
        } else {
            "garlic-dns".into()
        },
        jobs,
        authority,
        network: crate::network::Network::from_env(&state)?,
        exchanges: Default::default(),
        hub: Default::default(),
        accounts: AccountsService::new(account_file, keys.clone(), owner),
        devices,
        require_device: env::var("GARLIC_REQUIRE_DEVICE_LEASE").as_deref() == Ok("1"),
    });
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let network_source = env::var("GARLIC_NETWORK_UPSTREAM")
        .ok()
        .filter(|u| !u.is_empty() && u.trim_end_matches('/') != public.trim_end_matches('/'));
    let mut upstreams = std::collections::BTreeMap::new();
    if let Some(origin) = account_source {
        upstreams.entry(origin).or_insert((false, false)).0 = true;
    }
    if let Some(origin) = network_source {
        upstreams.entry(origin).or_insert((false, false)).1 = true;
    }
    for (origin, (accounts, network)) in upstreams {
        let ca = fs::read(path("GARLIC_CA_CERT")?)?;
        let id = env::var_os("GARLIC_NODE_IDENTITY_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| state.with_extension("identity.json"));
        let label = env::var("GARLIC_NODE_LABEL").unwrap_or_else(|_| "coordinator".into());
        let upstream_keys = keys.clone();
        let upstream = tokio::task::spawn_blocking(move || {
            crate::upstream::Upstream::new(
                &origin,
                &ca,
                id,
                upstream_keys,
                label,
                accounts,
                network,
            )
        })
        .await??;
        let shared = app.clone();
        let signal = stop.clone();
        std::thread::spawn(move || upstream.run(shared, signal));
    }
    if role == Role::Storage {
        use garlic_relay_session::endpoint::NodeEndpoint;
        use garlic_relay_session::runner::{self, WorkerSettings};
        let configured_local = env::var("GARLIC_LOCAL_STORAGE_URL").ok();
        let local_origin = configured_local.as_deref().unwrap_or(&public);
        let endpoint = if configured_local.is_some() {
            NodeEndpoint::new(local_origin)?
        } else {
            NodeEndpoint::bound(local_origin, listener.local_addr()?)?
        };
        let local = WorkerSettings::for_endpoint(endpoint)?;
        let remote = if env::var("GARLIC_DIRECT_ONLY").as_deref() == Ok("1") {
            None
        } else {
            Some(WorkerSettings::load()?).filter(|worker| {
                worker.endpoint.url().as_str().trim_end_matches('/')
                    != local_origin.trim_end_matches('/')
            })
        };
        let run = |worker: WorkerSettings| {
            let endpoint = worker.endpoint.clone();
            let service_stop = stop.clone();
            std::thread::spawn(move || {
                if let Err(e) = crate::queue::executor::run(endpoint, service_stop) {
                    eprintln!("Task executor stopped: {e}");
                }
            });
            let signal = stop.clone();
            std::thread::spawn(move || {
                if let Err(e) = runner::run(worker, signal) {
                    eprintln!("Storage worker stopped: {e}");
                }
            });
        };
        run(local);
        if let Some(remote) = remote {
            run(remote);
        }
    }
    let maintenance = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            maintenance
                .hub
                .cleanup(crate::accounts::now().saturating_mul(1000));
        }
    });
    if let Some(path) = manifest {
        let owner = Arc::downgrade(&app);
        tokio::spawn(async move {
            let mut last_error = None;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let Some(app) = owner.upgrade() else { break };
                let path = path.clone();
                let result = tokio::task::spawn_blocking(move || app.modules.reload(&path)).await;
                match result {
                    Ok(Ok(changed)) => {
                        if changed { eprintln!("exchange modules reloaded"); }
                        last_error = None;
                    }
                    Ok(Err(e)) => {
                        let message = e.to_string();
                        if last_error.as_ref() != Some(&message) {
                            eprintln!("exchange module reload rejected: {message}");
                            last_error = Some(message);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
    let routes = http::routes()
        .layer(axum::middleware::from_fn_with_state(
            app.clone(),
            http::authority_headers,
        ))
        .with_state(app);
    println!(
        "Garlic {} listening on {}",
        if role == Role::Storage {
            "storage"
        } else {
            "coordinator"
        },
        listener.local_addr()?
    );
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        let (mut stream, socket_peer) = tokio::select! {
            result=listener.accept()=>result?,
            _=&mut shutdown=>{stop.store(true,std::sync::atomic::Ordering::Release);return Ok(());}
        };
        let acceptor = acceptor.clone();
        let routes = routes.clone();
        tokio::spawn(async move {
            let handshake = async {
                stream.set_nodelay(true)?;
                let peer =
                    crate::proxy_protocol::peer(&mut stream, socket_peer, trust_loopback_proxy)
                        .await?;
                Ok::<_, std::io::Error>((acceptor.accept(stream).await?, peer))
            };
            let Ok(Ok((tls, peer))) =
                tokio::time::timeout(std::time::Duration::from_secs(15), handshake).await
            else {
                return;
            };
            let service = TowerToHyperService::new(
                routes.layer(axum::Extension(axum::extract::ConnectInfo(peer))),
            );
            let _ = Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(TokioIo::new(tls), service)
                .await;
        });
    }
}

#[cfg(unix)]
async fn shutdown_signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to subscribe to SIGTERM");
    tokio::select! {_=term.recv()=>{},_=tokio::signal::ctrl_c()=>{}}
}
#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
