use core::fmt::Debug;
use core::future::Future;
use drogue_ajour_protocol::{Command, Status};

/// Trait for the firmware update service.
///
/// The service is responsible for establishing the connection to the firmware update
/// service and performing the request-response cycle with the update service.
pub trait UpdateService {
    /// Error type
    type Error: core::fmt::Debug;

    /// Future returned by send
    type RequestFuture<'m>: Future<Output = Result<Command<'m>, Self::Error>> + 'm
    where
        Self: 'm;

    /// Send the status to the server, and return the Command responded by the service
    /// rx buffer.
    fn request<'m>(&'m mut self, status: &'m Status<'m>) -> Self::RequestFuture<'m>;
}

/// Type representing the firmware version
#[cfg(feature = "defmt")]
pub trait FirmwareVersion: PartialEq + AsRef<[u8]> + Sized + Debug + Clone + defmt::Format {
    fn from_slice(data: &[u8]) -> Result<Self, ()>;
}

#[cfg(not(feature = "defmt"))]
pub trait FirmwareVersion: PartialEq + AsRef<[u8]> + Sized + Debug + Clone {
    fn from_slice(data: &[u8]) -> Result<Self, ()>;
}

impl<const N: usize> FirmwareVersion for heapless::Vec<u8, N> {
    fn from_slice(data: &[u8]) -> Result<Self, ()> {
        heapless::Vec::from_slice(data)
    }
}

/// The current status of the firmware on a device
pub struct FirmwareStatus<VERSION>
where
    VERSION: FirmwareVersion,
{
    /// Current firmware version
    pub current_version: VERSION,
    /// Offset written of next firmware
    pub next_offset: u32,
    /// Next version being written
    pub next_version: Option<VERSION>,
}

pub trait FirmwareDevice {
    const MTU: usize;
    type Version: FirmwareVersion;
    type Error;

    // Future returned by status
    type StatusFuture<'m>: Future<Output = Result<FirmwareStatus<Self::Version>, Self::Error>> + 'm
    where
        Self: 'm;
    /// Return the status of the currently running firmware.
    fn status(&mut self) -> Self::StatusFuture<'_>;

    // Future returned by start
    type StartFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    /// Prepare for starting the firmware update process.
    fn start<'m>(&'m mut self, version: &'m [u8]) -> Self::StartFuture<'m>;

    /// Future returned by write
    type WriteFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    /// Write a block of firmware at the expected offset.
    fn write<'m>(&'m mut self, offset: u32, data: &'m [u8]) -> Self::WriteFuture<'m>;

    /// Future returned by update
    type UpdateFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    /// Finish the firmware write and mark device to be updated
    fn update<'m>(&'m mut self, version: &'m [u8], checksum: &'m [u8]) -> Self::UpdateFuture<'m>;

    /// Future returned by synced
    type SyncedFuture<'m>: Future<Output = Result<(), Self::Error>> + 'm
    where
        Self: 'm;
    /// Mark firmware as being in sync with the expected
    fn synced(&mut self) -> Self::SyncedFuture<'_>;
}
