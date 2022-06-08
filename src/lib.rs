//! Types for updating firmware of embedded devices from a remote server. The protocol is not
//! tied to any specific platform.
#![no_std]
#![cfg_attr(feature = "full", feature(generic_associated_types))]
#![cfg_attr(feature = "full", feature(type_alias_impl_trait))]

mod fmt;

mod protocol;
pub use protocol::*;

#[cfg(feature = "full")]
mod device;

#[cfg(feature = "full")]
pub use device::*;

#[cfg(feature = "full")]
mod service;

#[cfg(feature = "full")]
pub use service::*;

#[cfg(feature = "full")]
mod traits;

#[cfg(feature = "full")]
pub use traits::*;

#[cfg(feature = "full")]
mod updater;

#[cfg(feature = "full")]
pub use updater::*;
