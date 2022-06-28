//! Implementations of the `UpdateService` trait.
#[cfg(feature = "drogue")]
mod drogue;
mod memory;
mod serial;

#[cfg(feature = "drogue")]
pub use drogue::*;
pub use memory::*;
pub use serial::*;
