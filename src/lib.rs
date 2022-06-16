//! Types for updating firmware of embedded devices from a remote server. The protocol is not
//! tied to any specific platform.
#![no_std]
#![cfg_attr(feature = "nightly", feature(generic_associated_types))]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

mod fmt;

mod protocol;
pub use protocol::*;

#[cfg(feature = "nightly")]
pub mod device;

#[cfg(feature = "nightly")]
pub mod service;

#[cfg(feature = "nightly")]
mod traits;

#[cfg(feature = "nightly")]
pub use traits::*;

#[cfg(feature = "nightly")]
mod updater;

#[cfg(feature = "nightly")]
pub use updater::*;
