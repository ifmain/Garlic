use garlic_rollout::health;
use std::{env, process::ExitCode, thread};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let [slot, index] = args.as_slice() else {
        return 1.into();
    };
    let index: u16 = match index.as_str() {
        "1" => 1,
        "2" => 2,
        _ => return 2.into(),
    };
    let proxy = match slot.as_str() {
        "blue" => format!("socks5h://10.42.42.10:{}", 10800 + index),
        "green" => format!("socks5h://127.0.0.1:{}", 18300 + index),
        _ => return 2.into(),
    };
    let Some(ca) = health::certificate() else {
        return 1.into();
    };
    let result = thread::scope(|s| {
        let other = s.spawn(|| health::request(&ca, &proxy, "w.cs2u.ru", "5", true));
        let t1 = health::request(&ca, &proxy, "t1.cs2u.ru", "5", true);
        let w = other.join().unwrap_or(1);
        if w == 0 || t1 == 0 {
            0
        } else {
            w.max(t1)
        }
    });
    result.into()
}
