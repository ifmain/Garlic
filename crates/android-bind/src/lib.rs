#![cfg_attr(target_os = "android", no_std)]

#[cfg(any(target_os = "android", test))]
fn version_matches(reported: &[u8], required: &[u8]) -> bool {
    reported.split(|&c| c == b'\r' || c == b'\n').next() == Some(required)
}

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { abort() }
}

#[cfg(target_os = "android")]
#[link(name = "c")]
extern "C" {
    fn abort() -> !;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_exact_first_line_only() {
        for text in [b"0.9.2".as_slice(), b"0.9.2\nextra", b"0.9.2\r\nextra"] {
            assert!(version_matches(text, b"0.9.2"));
        }
        assert!(!version_matches(b"0.9.20", b"0.9.2"));
        assert!(!version_matches(b"0.9.2 ", b"0.9.2"));
    }
}
