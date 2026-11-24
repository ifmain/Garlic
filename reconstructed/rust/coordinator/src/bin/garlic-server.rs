use garlic_coordinator::{
    daemon::{serve, Role},
    gateway,
};
use garlic_relay_session::SessionError;

#[tokio::main]
async fn main() -> Result<(), SessionError> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: garlic-server --role entry|intermediate|exit|coordinator|gateway|xray-for-garlic [--grpc-route FILE]";
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--role")) {
        return Err(usage.into());
    }
    match args.next().and_then(|v| v.into_string().ok()).as_deref() {
        Some("coordinator" | "intermediate") if args.len() == 0 => serve(Role::Coordinator).await,
        Some("exit") if args.len() == 0 => serve(Role::Storage).await,
        Some("gateway" | "xray-for-garlic") => gateway::run(args).await,
        Some("entry") if args.len() == 0 => serve(Role::Coordinator).await,
        Some("entry") => {
            tokio::try_join!(serve(Role::Coordinator), gateway::run(args))?;
            Ok(())
        }
        _ => Err(usage.into()),
    }
}
