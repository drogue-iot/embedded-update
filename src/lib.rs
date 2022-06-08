//! Types for updating firmware of embedded devices from a remote server. The protocol is not
//! tied to any specific platform.
#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod fmt;

mod protocol;
pub use protocol::*;
mod device;
pub use device::*;
mod service;
pub use service::*;
mod traits;
pub use traits::*;
mod updater;
pub use updater::*;
