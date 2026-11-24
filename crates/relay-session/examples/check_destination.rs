use garlic_relay_session::destination::target_allowed;
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let ip = line?.parse()?;
        println!(
            "{} {}",
            target_allowed(ip, false) as u8,
            target_allowed(ip, true) as u8
        );
    }
    Ok(())
}
