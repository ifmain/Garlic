use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn certificate() -> Option<PathBuf> {
    let executable = env::current_exe().ok()?.canonicalize().ok()?;
    Some(executable.parent()?.parent()?.join("pki/ca.pem"))
}

pub fn classify(exit: Option<i32>, stdout: &[u8]) -> u8 {
    match exit {
        Some(0) => {
            let healthy = serde_json::from_slice::<serde_json::Value>(stdout)
                .ok()
                .and_then(|body| body.get("healthy")?.as_bool())
                == Some(true);
            if healthy {
                0
            } else {
                1
            }
        }
        Some(2 | 3 | 4 | 23 | 27 | 77) => 2,
        _ => 1,
    }
}

pub fn request(ca: &Path, proxy: &str, host: &str, timeout: &str, ipv4: bool) -> u8 {
    let mut curl = Command::new("/usr/bin/curl");
    if ipv4 {
        curl.arg("-4");
    }
    let result = curl
        .args([
            "--silent",
            "--fail",
            "--max-time",
            timeout,
            "--proxy",
            proxy,
            "--noproxy",
            "",
            "--cacert",
        ])
        .arg(ca)
        .arg(format!("https://{host}:8443/health"))
        .output();
    match result {
        Ok(output) => classify(output.status.code(), &output.stdout),
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_curl_and_response_body() {
        assert_eq!(classify(Some(0), br#"{"healthy":true}"#), 0);
        for body in [
            br#"{"healthy":1}"#.as_slice(),
            br#"{"healthy":"true"}"#,
            b"[]",
            b"broken",
        ] {
            assert_eq!(classify(Some(0), body), 1);
        }
        for n in [2, 3, 4, 23, 27, 77] {
            assert_eq!(classify(Some(n), b""), 2);
        }
        assert_eq!(classify(Some(28), b""), 1);
    }
}
