use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_release_cache::guest::Guest;
#[test]
fn guest_budget_stops_nonterminating_code() {
    // loop { br 0 }; the trailing constant satisfies the return type.
    let bytes = b"\0asm\x01\0\0\0\x01\x08\x01\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x02\x01\0\x07\x05\x01\x01f\0\0\x0a\x0b\x01\x09\0\x03\x40\x0c\0\x0b\x41\0\x0b";
    let guest=Guest::load(&STANDARD.encode(bytes)).unwrap();
    assert!(guest.call("f",(0,0,0)).is_err());
}
