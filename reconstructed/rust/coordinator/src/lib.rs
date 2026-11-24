//! Coordinator components reconstructed against the deployed Garlic protocol.
pub mod accounts;
pub mod devices;
pub mod exchanges;
pub mod files;
pub mod http;
pub mod hub;
mod legacy_panel;
pub mod network;
pub mod queue;
pub mod streams;
pub mod upstream;
pub mod worker;

pub mod daemon;
mod proxy_protocol;
pub mod updates;

#[path = "bin/garlic-xui-legacy/mod.rs"]
pub mod gateway;
