# Development snapshot — not Garlic v1.0

Based on user-supplied Garlic v0.2 source. Rust 1.99.1; dependencies frozen in Cargo.lock.

Verified on Linux x86_64: release build; 3 unit tests; integration tests for direct execution, Infinity queue, binary HTTP roundtrip, upstream 418 status, coordinator restart, deduplication, conflict and unauthorized access. Android ARM64: cargo check of garlic-core only. No Android APK or VPN implementation.

Changes: bounded incremental upstream response reads; external plaintext binds require explicit development opt-in; ambiguous coordinator submission does not fail over; mutating HTTP tasks do not automatically replay after lease expiration or coordinator restart; completion replay checks owner, lease and result.

Outstanding mandatory SRS scope: transactional SQL persistence and rollback on failed commit, durable client journal, protocol v1/task deadlines/client ownership/role separation, cancellation/renewal, registration and service-aware scheduling, aging/fairness, quotas and complete resource bounds, adaptive degradation guard, production TLS setup, portable client library and full CLI, performance/failure/security acceptance suites. Current JSON snapshot storage is not a transactional queue. Failed mutations may have been executed upstream and require manual reconciliation. Pending mutations not yet leased may be executed after restart.

This snapshot is not a system VPN. System traffic transport requires a separate design (e.g. WireGuard, Windows adapter/service and Android VpnService); it is explicitly outside the attached SRS v1 HTTP scope. Do not use this snapshot for production.

Run from repository root:

    export CARGO_HOME=/workspace/toolchains/cargo
    export RUSTUP_HOME=/workspace/toolchains/rustup
    export PATH="$CARGO_HOME/bin:$PATH"
    cargo test --workspace --locked
    cargo build --release --workspace --locked
    python integration-tests/smoke.py

See README.md for local launch variables. Use loopback and a TLS/authentication proxy for deployment experiments. All runtime processes must restart after environment restore.

## HTTPS/HTTP2 development transport

See PROXY.md. Added garlic-proxy (entry/exit and local CONNECT client), TLS via Caddy, DoH tunnel and Cloudflare/Google fallback, exit list and bootstrap IP with hostname verification. Original SRS gaps above remain. Exact Chromium fingerprint, Android APK and OS integration remain unimplemented.
