use garlic_rollout::health;
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let [port, host] = args.as_slice() else {
        return 1.into();
    };
    let Some(ca) = health::certificate().filter(|p| p.is_file()) else {
        return 2.into();
    };
    let Ok(port) = port.trim().parse::<i64>() else {
        return 1.into();
    };
    health::request(&ca, &format!("http://127.0.0.1:{port}"), host, "8", true).into()
}
