use crate::traits::{FirmwareDevice, Status};
use core::convert::Infallible;
use core::future::Future;
use heapless::Vec;

/// A simulated device which implements the `FirmwareDevice` trait.
pub struct Simulator {
    version: Vec<u8, 16>,
}

impl Simulator {
    pub fn new(version: &[u8]) -> Self {
        Self {
            version: Vec::from_slice(version).unwrap(),
        }
    }
}

impl FirmwareDevice for Simulator {
    const MTU: usize = 8;
    type Error = Infallible;

    type StatusFuture<'m> = impl Future<Output = Result<Status<'m>, Self::Error>> + 'm
    where
        Self: 'm;
    fn status<'m>(&'m mut self) -> Self::StatusFuture<'m> {
        async move {
            Ok(Status {
                current_version: &self.version,
                next_offset: 0,
                next_version: None,
            })
        }
    }

    type StartFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn start<'m>(&'m mut self, _: &'m [u8]) -> Self::StartFuture<'m> {
        async move { Ok(()) }
    }

    type WriteFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn write<'m>(&'m mut self, _: u32, _: &'m [u8]) -> Self::WriteFuture<'m> {
        async move { Ok(()) }
    }

    type UpdateFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn update<'m>(&'m mut self, version: &'m [u8], _: [u8; 32]) -> Self::UpdateFuture<'m> {
        async move {
            self.version = Vec::from_slice(version).unwrap();
            Ok(())
        }
    }

    type SyncedFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn synced<'m>(&'m mut self) -> Self::SyncedFuture<'m> {
        async move { Ok(()) }
    }
}
