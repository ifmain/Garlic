use garlic_relay_data::control::{Authority, StreamOpen};
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let (open, authority, now): (StreamOpen, Authority, u64) = serde_json::from_str(&line?)?;
        println!(
            "{}",
            serde_json::to_string(&open.validate(&authority, now).err())?
        );
    }
    Ok(())
}
