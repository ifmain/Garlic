# From the workspace root, with DNS and Storage already running.
$ErrorActionPreference = 'Stop'
cargo test --workspace
cargo check --workspace
cargo run -p garlic-internal-network -- uppercase 'hello garlic'
