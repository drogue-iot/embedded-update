//! Implementations of the `FirmwareDevice` trait.
mod serial;
mod simulator;

pub use serial::*;
pub use simulator::*;
