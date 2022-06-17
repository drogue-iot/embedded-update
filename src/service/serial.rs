use core::future::Future;
use embedded_io::asynch::{Read, Write};
use postcard::{from_bytes, to_slice};

use crate::protocol::{Command, Status};
use crate::traits::UpdateService;

/// Defines a fixed frame protocol based on types
pub const FRAME_SIZE: usize = 1024;

/// An update service based on a fixed-frame serial protocol, using `postcard` as the serialization format.
/// Can be used with any transport implementing the embedded-io traits. (TCP, UDP, UART, USB).
pub struct Serial<T>
where
    T: Read + Write,
{
    transport: T,
    buf: [u8; FRAME_SIZE],
}

impl<T> Serial<T>
where
    T: Read + Write,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            buf: [0; FRAME_SIZE],
        }
    }
}

#[derive(Debug)]
pub enum SerialError<T, C> {
    Transport(T),
    Codec(C),
}

impl<T> UpdateService for Serial<T>
where
    T: Read + Write,
{
    type Error = SerialError<T::Error, postcard::Error>;

    type RequestFuture<'m> = impl Future<Output = Result<Command<'m>, Self::Error>> + 'm where Self: 'm;

    fn request<'m>(&'m mut self, status: &'m Status<'m>) -> Self::RequestFuture<'m> {
        async move {
            to_slice(&status, &mut self.buf).map_err(|e| SerialError::Codec(e))?;
            let _ = self
                .transport
                .write(&self.buf)
                .await
                .map_err(|e| SerialError::Transport(e))?;

            let _ = self
                .transport
                .read(&mut self.buf)
                .await
                .map_err(|e| SerialError::Transport(e))?;

            let c: Command = from_bytes(&self.buf).map_err(|e| SerialError::Codec(e))?;
            Ok(c)
        }
    }
}
