#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod fmt;

mod device;
pub use device::*;
mod service;
pub use service::*;
mod traits;
pub use traits::*;
mod updater;
pub use updater::*;
