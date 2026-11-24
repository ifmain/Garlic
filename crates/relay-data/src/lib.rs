//! Wire representations shared by the Garlic relay coordinator and exit worker.
pub mod control;
mod data;
pub mod envelope;
pub mod exchange;
pub mod gex4;
pub mod stream;
pub use data::{decode, Direction, Frame, InvalidFrame, MAX_PAYLOAD};
