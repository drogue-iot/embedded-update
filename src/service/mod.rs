//! Implementations of the `UpdateService` trait.
#[cfg(feature = "drogue")]
mod drogue;
#[cfg(feature = "hawkbit")]
mod hawkbit;
mod memory;
mod serial;

#[cfg(feature = "drogue")]
pub use drogue::*;
#[cfg(feature = "hawkbit")]
pub use hawkbit::*;
pub use memory::*;
pub use serial::*;
