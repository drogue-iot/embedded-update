use {
    crate::traits::{FirmwareDevice, FirmwareStatus},
    core::convert::Infallible,
    heapless::Vec,
};

/// A simulated device which implements the `FirmwareDevice` trait.
pub struct Simulator {
    version: Vec<u8, 16>,
}

impl Simulator {
    /// Create a new instance of a simulated device with a given version.
    pub fn new(version: &[u8]) -> Self {
        Self {
            version: Vec::from_slice(version).unwrap(),
        }
    }

    /// Return the current version of the device.
    pub fn version(&self) -> &[u8] {
        &self.version[..]
    }
}

impl FirmwareDevice for Simulator {
    const MTU: usize = 256;
    type Version = Vec<u8, 16>;
    type Error = Infallible;

    async fn status(&mut self) -> Result<FirmwareStatus<Self::Version>, Self::Error> {
        debug!("Simulator::status()");
        Ok(FirmwareStatus {
            current_version: self.version.clone(),
            next_offset: 0,
            next_version: None,
        })
    }

    async fn start(&mut self, _version: &[u8]) -> Result<(), Self::Error> {
        debug!("Simulator::start()");
        Ok(())
    }

    async fn write(&mut self, _offset: u32, _data: &[u8]) -> Result<(), Self::Error> {
        debug!("Simulator::write()");
        Ok(())
    }

    async fn update(&mut self, version: &[u8], _checksum: &[u8]) -> Result<(), Self::Error> {
        debug!("Simulator::update()");
        self.version = Vec::from_slice(version).unwrap();
        Ok(())
    }

    async fn synced(&mut self) -> Result<(), Self::Error> {
        debug!("Simulator::synced()");
        Ok(())
    }
}
