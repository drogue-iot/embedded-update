#![no_std]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

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
