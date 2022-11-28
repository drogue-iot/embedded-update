use {
    crate::protocol::{Command, Status},
    core::fmt::Debug,
};

/// Trait for the firmware update service.
///
/// The service is responsible for establishing the connection to the firmware update
/// service and performing the request-response cycle with the update service.
pub trait UpdateService {
    /// Error type
    type Error: core::fmt::Debug;

    /// Send the status to the server, and return the Command responded by the service
    /// rx buffer.
    async fn request<'m>(&'m mut self, status: &'m Status<'m>) -> Result<Command<'m>, Self::Error>;
}

/// Type representing the firmware version
#[cfg(feature = "defmt")]
pub trait FirmwareVersion: PartialEq + AsRef<[u8]> + Sized + Debug + Clone + defmt::Format {
    /// Create an instance of the version based on a byte slice
    fn from_slice(data: &[u8]) -> Result<Self, ()>;
}

/// Type representing the firmware version
#[cfg(not(feature = "defmt"))]
pub trait FirmwareVersion: PartialEq + AsRef<[u8]> + Sized + Debug + Clone {
    /// Create an instance of the version based on a byte slice
    #[allow(clippy::result_unit_err)]
    fn from_slice(data: &[u8]) -> Result<Self, ()>;
}

impl<const N: usize> FirmwareVersion for heapless::Vec<u8, N> {
    fn from_slice(data: &[u8]) -> Result<Self, ()> {
        heapless::Vec::from_slice(data)
    }
}

#[cfg(feature = "std")]
mod stdlib {
    extern crate std;
    use std::vec::Vec;
    impl super::FirmwareVersion for Vec<u8> {
        fn from_slice(data: &[u8]) -> Result<Self, ()> {
            Ok(data.into())
        }
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

impl<VERSION> Clone for FirmwareStatus<VERSION>
where
    VERSION: FirmwareVersion + Clone,
{
    fn clone(&self) -> Self {
        Self {
            current_version: self.current_version.clone(),
            next_offset: self.next_offset,
            next_version: self.next_version.clone(),
        }
    }
}

/// Represents a device that can be updated by a `FirmwareUpdater`.
pub trait FirmwareDevice {
    /// The preferred block size to be passed in write.
    const MTU: usize;

    /// The expected version type for this device.
    type Version: FirmwareVersion;

    /// The error type.
    type Error;

    /// Return the status of the currently running firmware.
    async fn status(&mut self) -> Result<FirmwareStatus<Self::Version>, Self::Error>;

    /// Prepare for starting the firmware update process.
    async fn start(&mut self, version: &[u8]) -> Result<(), Self::Error>;

    /// Write a block of firmware at the expected offset.
    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error>;

    /// Finish the firmware write and mark device to be updated
    async fn update(&mut self, version: &[u8], checksum: &[u8]) -> Result<(), Self::Error>;

    /// Mark firmware as being in sync with the expected
    async fn synced(&mut self) -> Result<(), Self::Error>;
}
