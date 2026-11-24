# Garlic Alpha

[Android and Windows downloads](https://github.com/ifmain/Garlic/releases/tag/alpha-0.9.2.1)

The Alpha client and server sources live under [reconstructed](reconstructed). The Windows UI is in [windows](reconstructed/windows), the Android launcher and service in [android-apk](reconstructed/android-apk), and the Alpha UI changes and packager in [alpha/android](reconstructed/alpha/android). The shared client, protocol codecs and server roles are in the [Rust workspace](reconstructed/rust). See the [module ABI](reconstructed/alpha/protocol-module.md) for the supported instructions and update procedure.

### Building Alpha

Run Cargo in `reconstructed/rust`. For a Linux server use `cargo build --locked --release -p garlic-coordinator --bin garlic-server`. For Windows x64 build `cargo build --locked --release --target x86_64-pc-windows-gnullvm -p garlic-client --features alpha --bin garlic-internal-network`, then run `reconstructed/windows/build.ps1 -Alpha -Core <absolute-path-to-core.exe>` on Windows.

For Android, configure the NDK clang linker for API 29 for each target, and build `cargo build --locked --release --target <target> -p garlic-client --features alpha --lib` for both `aarch64-linux-android` and `armv7-linux-androideabi`. The packager expects a JDK in `.toolchains/jdk-*/bin`, Android SDK platform 35 and build-tools 35.0.0 in `.toolchains/android-sdk`, and `.toolchains/apktool_3.0.3.jar`. From the repository root run `python reconstructed/alpha/android/build.py garlic-alpha.apk`. Local builds generate a local preview signing key. Android requires the same signer to install an update over an existing Alpha installation.

### Previous releases

## Garlic v0.3.0 — HTTPS/HTTP2 proxy prototype

Garlic provides an authenticated proxy transport over HTTPS and HTTP/2, with an entry node, one or more exit nodes, a local CONNECT client, and DNS forwarding inside the channel. The original Infinity HTTP task queue remains a separate component.

**Development prerelease.** Linux builds, unit tests and local TLS/HTTP2 integration tests passed. Windows x64 binaries were cross-compiled; Windows execution and public Internet egress were not validated in the development environment. No Android APK or Chromium-compatible outer TLS fingerprint is included. This is not a full implementation of the original v1.0 specification.

## Downloads and instructions

See [GitHub Releases](https://github.com/ifmain/Garlic/releases) for published prereleases. The tag workflow builds Linux x64 and Windows x64 archives, a source archive, and SHA256 checksums from the tagged commit. A pushed tag alone does not establish a successful release; check the workflow and attached assets.

- [Proxy deployment and client configuration (Russian)](docs/PROXY.md)
- [Known limitations and remaining requirements](docs/STATUS.md)
- [Infinity HTTP task-exchange guide](docs/TASK_EXCHANGE.md)
- [Release notes](docs/releases/v0.3.0.md)
- [Caddy HTTPS / Let's Encrypt configuration](configs/Caddyfile)

## Build and test

Install Rust using [rustup](https://rustup.rs/); `rust-toolchain.toml` pins Rust 1.99.1. Native Windows builds also need the Microsoft C++ build tools.

```sh
cargo test --workspace --locked
cargo build --release --workspace --locked
python integration-tests/smoke.py
```

For the additional TLS/HTTP2 integration test, install OpenSSL, Python package `h2==4.4.1`, and a verified Caddy 2.10.2 binary:

```sh
python -m pip install h2==4.4.1
CADDY=/path/to/caddy python integration-tests/proxy_h2.py
```

This test creates a temporary CA and leaf certificate. It exercises TLS verification, ALPN h2, authentication, HTTP/1 rejection, full-duplex relay against a mock exit, a local DNS stub, and exit DNS fallback against mock resolvers. It does not validate a deployed foreign exit or production ACME issuance.

## Run the proxy

Use the deployment guide for complete settings. Supply keys through protected environment variables; do not commit them.

- Exit server: set `GARLIC_PROXY_KEY`; run `garlic-proxy server` behind Caddy.
- Entry server: set `GARLIC_PROXY_KEY`, `GARLIC_EXIT_KEY`, and `GARLIC_EXIT_URLS`; run `garlic-proxy server` behind Caddy.
- Client: set `GARLIC_PROXY_KEY` and `GARLIC_ENTRY_URL`; run `garlic-proxy client` (`garlic-proxy.exe client` on Windows).

The client exposes HTTP CONNECT on `127.0.0.1:1080` and a UDP DNS stub on `127.0.0.1:1053`. Configure applications explicitly. This does not redirect all device traffic or automatically change system DNS. Only public TCP port 443 targets are permitted. Cloudflare DoH is primary, Google DoH is fallback on the exit node. Actual certificate issuance requires your own domain and reachable ACME endpoints.

## Release process

Maintainers push an annotated `v*` tag after validation. [.github/workflows/release.yml](.github/workflows/release.yml) tests and builds both platforms, packages artifacts with [scripts/package-release.py](scripts/package-release.py), and publishes a GitHub prerelease using the workflow's scoped `GITHUB_TOKEN`. No personal token is stored in the repository. Publication requires GitHub Actions to be enabled and to allow the workflow `contents: write` permission.
