#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), garlic_relay_session::SessionError> {
    garlic_coordinator::gateway::run(std::env::args_os().skip(1)).await
}
