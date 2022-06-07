use crate::traits::{FirmwareDevice, FirmwareStatus};
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
    const MTU: usize = 256;
    type Version = Vec<u8, 16>;
    type Error = Infallible;

    type StatusFuture<'m> = impl Future<Output = Result<FirmwareStatus<Self::Version>, Self::Error>> + 'm
    where
        Self: 'm;
    fn status(&mut self) -> Self::StatusFuture<'_> {
        async move {
            debug!("Simulator::status()");
            Ok(FirmwareStatus {
                current_version: self.version.clone(),
                next_offset: 0,
                next_version: None,
            })
        }
    }

    type StartFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn start<'m>(&'m mut self, _: &'m [u8]) -> Self::StartFuture<'m> {
        async move {
            debug!("Simulator::start()");
            Ok(())
        }
    }

    type WriteFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn write<'m>(&'m mut self, _: u32, _: &'m [u8]) -> Self::WriteFuture<'m> {
        async move {
            debug!("Simulator::write()");
            Ok(())
        }
    }

    type UpdateFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn update<'m>(&'m mut self, version: &'m [u8], _: &'m [u8]) -> Self::UpdateFuture<'m> {
        async move {
            debug!("Simulator::update()");
            self.version = Vec::from_slice(version).unwrap();
            Ok(())
        }
    }

    type SyncedFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    fn synced(&mut self) -> Self::SyncedFuture<'_> {
        async move {
            debug!("Simulator::synced()");
            Ok(())
        }
    }
}
