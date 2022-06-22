use crate::protocol::{Command, Status};
use core::convert::Infallible;
use core::future::Future;

use crate::traits::UpdateService;

/// An in-memory updater service, useful in tests.
pub struct InMemory<'a> {
    expected_version: &'a [u8],
    expected_firmware: &'a [u8],
}

impl<'a> InMemory<'a> {
    /// Create a new inmemory update service with a version and firmare.
    pub fn new(expected_version: &'a [u8], expected_firmware: &'a [u8]) -> Self {
        Self {
            expected_version,
            expected_firmware,
        }
    }
}

impl<'a> UpdateService for InMemory<'a> {
    type Error = Infallible;

    type RequestFuture<'m> = impl Future<Output = Result<Command<'m>, Self::Error>> + 'm where Self: 'm;
    fn request<'m>(&'m mut self, status: &'m Status<'m>) -> Self::RequestFuture<'m> {
        async move {
            if self.expected_version == status.version.as_ref() {
                Ok(Command::new_sync(self.expected_version, None, status.correlation_id))
            } else if let Some(update) = &status.update {
                if update.version == self.expected_version {
                    if update.offset as usize >= self.expected_firmware.len() {
                        // Update is finished, instruct device to swap
                        Ok(Command::new_swap(self.expected_version, &[], status.correlation_id))
                    } else {
                        // Continue updating
                        let data = self.expected_firmware;
                        let mtu = status.mtu.unwrap_or(16) as usize;
                        let to_copy = core::cmp::min(mtu, data.len() - update.offset as usize);
                        let s = &data[update.offset as usize..update.offset as usize + to_copy];
                        Ok(Command::new_write(
                            self.expected_version,
                            update.offset,
                            s,
                            status.correlation_id,
                        ))
                    }
                } else {
                    //  Unexpected version in status update, we need to start at 0
                    let data = self.expected_firmware;
                    let mtu = status.mtu.unwrap_or(128) as usize;
                    let to_copy = core::cmp::min(mtu, data.len());
                    let s = &data[..to_copy];
                    Ok(Command::new_write(self.expected_version, 0, s, status.correlation_id))
                }
            } else {
                // No update status, start a new update
                let data = self.expected_firmware;
                let mtu = status.mtu.unwrap_or(128) as usize;
                let to_copy = core::cmp::min(mtu, data.len());
                let s = &data[..to_copy];
                Ok(Command::new_write(self.expected_version, 0, s, status.correlation_id))
            }
        }
    }
}
