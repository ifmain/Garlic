use std::sync::Mutex;

/// Android's ConnectionState tracks whether an asynchronous connection request is current.
#[derive(Default)]
pub struct ConnectionState(Mutex<Generation>);

#[derive(Default)]
struct Generation {
    active: bool,
    desired: bool,
    number: i64,
}

impl ConnectionState {
    pub fn begin(&self) -> i64 {
        let mut state = self.0.lock().unwrap();
        state.desired = true;
        if state.active {
            return -1;
        }
        state.active = true;
        state.number = state.number.wrapping_add(1);
        state.number
    }

    pub fn current(&self, generation: i64) -> bool {
        let state = self.0.lock().unwrap();
        state.desired && state.active && state.number == generation
    }

    pub fn invalidate(&self) {
        let mut state = self.0.lock().unwrap();
        state.active = false;
        state.number = state.number.wrapping_add(1);
    }

    pub fn stop(&self) {
        let mut state = self.0.lock().unwrap();
        state.desired = false;
        state.active = false;
        state.number = state.number.wrapping_add(1);
    }
}

pub fn retry_allowed(message: Option<&str>) -> bool {
    let Some(message) = message else { return true };
    let lower = message.to_lowercase();
    ![
        "device_in_use",
        "device access revoked",
        "http 401",
        "http 402",
        "http 403",
        "http 409",
        "vpn permission is required",
        "сетевой модуль недоступен",
    ]
    .iter()
    .any(|reason| lower.contains(reason))
}

/// The two literal fragments are recovered from the native core's format string.
pub fn generate_pac(proxy_address: &std::net::SocketAddr) -> String {
    include_str!("proxy.pac.template").replace("{proxy_address}", &proxy_address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_socket_display() {
        for address in ["127.0.0.1:1080", "[::1]:8080"] {
            let pac = generate_pac(&address.parse().unwrap());
            assert!(pac.ends_with(&format!("return \"PROXY {address}; DIRECT\";\n}}\n")));
        }
    }

    #[test]
    fn duplicate_and_stale_requests() {
        let state = ConnectionState::default();
        assert_eq!(state.begin(), 1);
        assert_eq!(state.begin(), -1);
        assert!(state.current(1));
        state.invalidate();
        assert!(!state.current(1));
        assert_eq!(state.begin(), 3);
        state.stop();
        assert!(!state.current(3));
        assert_eq!(state.begin(), 5);
    }

    #[test]
    fn retry_policy() {
        assert!(retry_allowed(None));
        assert!(retry_allowed(Some("HTTP 503")));
        assert!(!retry_allowed(Some("Error: HTTP 409")));
        assert!(!retry_allowed(Some("СЕТЕВОЙ МОДУЛЬ НЕДОСТУПЕН")));
    }
}
