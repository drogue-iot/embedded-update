//! Implementations of the `UpdateService` trait.
mod memory;
mod serial;

pub use {memory::*, serial::*};
