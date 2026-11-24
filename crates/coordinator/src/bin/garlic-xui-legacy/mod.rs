mod local_vless;
mod socks;
mod sync;
mod trojan_grpc;

use serde::Deserialize;
use std::error::Error;

type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize)]
struct Config {
    s1: local_vless::Route,
    s2: local_vless::Route,
    trojan_grpc: Option<trojan_grpc::Settings>,
}

pub async fn run(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result {
    let first = args.next().ok_or("expected gateway configuration")?;
    if first == "--grpc-route" {
        #[derive(Deserialize)]
        struct SingleRoute {
            route: local_vless::Route,
            trojan_grpc: trojan_grpc::Settings,
        }
        let path = args.next().ok_or("expected route configuration")?;
        let config: SingleRoute = serde_json::from_slice(&std::fs::read(path)?)?;
        let proxy = config.route.garlic_proxy;
        tokio::try_join!(
            local_vless::listen(config.route),
            trojan_grpc::listen(Some(config.trojan_grpc), proxy),
        )?;
        return Ok(());
    }
    let config: Config = serde_json::from_slice(&std::fs::read(first)?)?;
    let trojan_proxy = config.s1.garlic_proxy;
    tokio::try_join!(
        local_vless::listen(config.s1),
        local_vless::listen(config.s2),
        trojan_grpc::listen(config.trojan_grpc, trojan_proxy),
    )?;
    Ok(())
}
