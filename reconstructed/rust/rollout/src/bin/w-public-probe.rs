use garlic_rollout::health;
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let Some(arg) = env::args().nth(1) else {
        return 1.into();
    };
    let Ok(index) = arg.trim().parse::<i32>() else {
        return 1.into();
    };
    if !(1..=2).contains(&index) {
        return 2.into();
    }
    let Some(ca) = health::certificate() else {
        return 1.into();
    };
    let proxy = format!("socks5h://127.0.0.1:{}", 18300 + index);
    let healthy = |host| health::request(&ca, &proxy, host, "4", false) == 0;
    if healthy("w.cs2u.ru") {
        return 0.into();
    }
    if healthy(&format!("s{index}.cs2u.ru")) {
        1.into()
    } else {
        2.into()
    }
}
