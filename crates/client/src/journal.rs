use chrono::{DateTime, NaiveDate, Utc};
use std::{fs, io::Write, path::Path, sync::Mutex, time::{Instant, SystemTime}};

static LAST_ERROR: Mutex<Option<(&'static str, Instant)>> = Mutex::new(None);

pub fn reason(message: &str) -> &'static str {
    let text = message.to_ascii_lowercase();
    for (needle, label) in [
        ("401", "unauthorized"), ("403", "forbidden"), ("409", "conflict"),
        ("429", "rate_limited"), ("503", "service_unavailable"), ("capacity", "capacity"),
        ("timeout", "timeout"), ("timed out", "timeout"), ("reset", "connection_reset"),
        ("eof", "unexpected_eof"), ("dns", "dns"), ("firewall", "firewall"),
        ("session", "session"), ("route", "routing"), ("certificate", "tls_certificate"),
        ("wintun", "wintun"), ("refused", "connection_refused"), ("permission", "permission"),
    ] { if text.contains(needle) { return label; } }
    "unclassified"
}

pub fn write(config: &Path, event: &'static str, detail: &str) {
    if let Ok(mut last) = LAST_ERROR.lock() {
        if event.ends_with("_error") {
            if last.is_some_and(|(e, t)| e == event && t.elapsed().as_secs() < 1) { return; }
            *last = Some((event, Instant::now()));
        }
        let Some(parent) = config.parent() else { return };
        let dir = parent.join("logs");
        let now: DateTime<Utc> = SystemTime::now().into();
        if fs::create_dir_all(&dir).is_err() { return; }
        let file = dir.join(format!("{}.log", now.date_naive()));
        if event == "proxy_start" || !file.exists() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("log") { continue; }
                    if let Some(date) = path.file_stem().and_then(|s| s.to_str())
                        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()) {
                        if (now.date_naive() - date).num_days() >= 7 { let _ = fs::remove_file(path); }
                    }
                }
            }
        }
        if fs::metadata(&file).is_ok_and(|m| m.len() >= 8 * 1024 * 1024) { return; }
        if let Ok(mut out) = fs::OpenOptions::new().create(true).append(true).open(file) {
            let _ = writeln!(out, "{} {} {}", now.format("%FT%T%.3fZ"), event, detail);
        }
    }
}
