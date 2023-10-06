use {
    crate::{
        protocol::*,
        traits::{FirmwareDevice, FirmwareStatus},
    },
    embedded_io_async::{Read, Write},
    heapless::Vec,
    postcard::{from_bytes, to_slice},
};

/// Defines a fixed frame protocol based on types
const FRAME_SIZE: usize = 1024;

/// A FirmwareDevice based on a fixed-frame serial protocol, using `postcard` as the serialization format.
/// Can be used with any transport implementing the embedded-io traits. (TCP, UDP, UART, USB).
pub struct Serial<T>
where
    T: Read + Write,
{
    status: FirmwareStatus<Vec<u8, 16>>,
    transport: T,
    buf: [u8; FRAME_SIZE],
}

impl<T> Serial<T>
where
    T: Read + Write,
{
    /// Create a Serial instance using the provided transport.
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            buf: [0; FRAME_SIZE],
            status: FirmwareStatus {
                current_version: Vec::new(),
                next_version: None,
                next_offset: 0,
            },
        }
    }
}

/// Errors returned by Serial
#[derive(Debug)]
pub enum SerialError<T, C> {
    /// An error from the underlying transport layer
    Transport(T),
    /// An error during encode/decode of the status/command payload
    Codec(C),
    /// Other internal error.
    Other,
}

impl<T> FirmwareDevice for Serial<T>
where
    T: Read + Write,
{
    const MTU: usize = 968;
    type Version = Vec<u8, 16>;
    type Error = SerialError<T::Error, postcard::Error>;

    async fn status(&mut self) -> Result<FirmwareStatus<Self::Version>, Self::Error> {
        let _ = self
            .transport
            .read(&mut self.buf)
            .await
            .map_err(SerialError::Transport)?;

        let status: Status = from_bytes(&self.buf).map_err(SerialError::Codec)?;
        self.status.current_version = Vec::from_slice(&status.version).map_err(|_| SerialError::Other)?;
        if let Some(update) = status.update {
            self.status.next_offset = update.offset;
            self.status
                .next_version
                .replace(Vec::from_slice(&update.version).map_err(|_| SerialError::Other)?);
        }
        Ok(self.status.clone())
    }

    async fn start(&mut self, version: &[u8]) -> Result<(), Self::Error> {
        self.status.next_offset = 0;
        self.status
            .next_version
            .replace(Vec::from_slice(version).map_err(|_| SerialError::Other)?);
        Ok(())
    }

    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        let command: Command = Command::new_write(self.status.next_version.as_ref().unwrap(), offset, data, None);
        to_slice(&command, &mut self.buf).map_err(SerialError::Codec)?;
        let _ = self.transport.write(&self.buf).await.map_err(SerialError::Transport)?;
        Ok(())
    }

    async fn update(&mut self, version: &[u8], checksum: &[u8]) -> Result<(), Self::Error> {
        let command: Command = Command::new_swap(version, checksum, None);
        to_slice(&command, &mut self.buf).map_err(SerialError::Codec)?;
        let _ = self.transport.write(&self.buf).await.map_err(SerialError::Transport)?;
        Ok(())
    }

    async fn synced(&mut self) -> Result<(), Self::Error> {
        let command: Command = Command::new_sync(&self.status.current_version, None, None);
        to_slice(&command, &mut self.buf).map_err(SerialError::Codec)?;
        let _ = self.transport.write(&self.buf).await.map_err(SerialError::Transport)?;
        Ok(())
    }
}
