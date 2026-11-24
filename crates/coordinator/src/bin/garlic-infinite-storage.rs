use garlic_coordinator::daemon::{serve, Role};
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), garlic_relay_session::SessionError> {
    serve(Role::Storage).await
}
