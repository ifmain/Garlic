use garlic_relay_session::{
    runner::{self, WorkerSettings},
    SessionError,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
fn main() -> Result<(), SessionError> {
    let settings = WorkerSettings::load()?;
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    runner::run(settings, stop)
}
