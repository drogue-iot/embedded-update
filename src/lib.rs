#![no_std]
#![feature(generic_associated_types)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

mod fmt;

mod traits;
pub use traits::*;
mod updater;
pub use updater::*;
