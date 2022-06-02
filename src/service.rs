use core::convert::Infallible;
use core::future::Future;
use drogue_ajour_protocol::{CommandRef, StatusRef};

use crate::traits::UpdateService;

/// An in-memory updater service, useful in tests.
pub struct InMemory<'a> {
    expected_version: &'a [u8],
    expected_firmware: &'a [u8],
}

impl<'a> InMemory<'a> {
    pub fn new(expected_version: &'a [u8], expected_firmware: &'a [u8]) -> Self {
        Self {
            expected_version,
            expected_firmware,
        }
    }
}

impl<'a> UpdateService for InMemory<'a> {
    type Error = Infallible;

    type RequestFuture<'m> = impl Future<Output = Result<CommandRef<'m>, Self::Error>> + 'm where Self: 'm;
    fn request<'m>(&'m mut self, status: &'m StatusRef<'m>) -> Self::RequestFuture<'m> {
        async move {
            if self.expected_version == status.version {
                Ok(CommandRef::Sync {
                    version: self.expected_version,
                    poll: None,
                    correlation_id: status.correlation_id,
                })
            } else if let Some(update) = &status.update {
                if update.version == self.expected_version {
                    if update.offset as usize == self.expected_firmware.len() {
                        // Update is finished, instruct device to swap
                        Ok(CommandRef::Swap {
                            version: self.expected_version,
                            correlation_id: status.correlation_id,
                            checksum: [0; 32],
                        })
                    } else {
                        // Continue updating
                        let data = self.expected_firmware;
                        let mtu = status.mtu.unwrap_or(128) as usize;
                        let to_copy = core::cmp::min(mtu, data.len() - update.offset as usize);
                        let s = &data[update.offset as usize..update.offset as usize + to_copy];
                        Ok(CommandRef::Write {
                            version: self.expected_version,
                            correlation_id: status.correlation_id,
                            offset: update.offset,
                            data: s,
                        })
                    }
                } else {
                    //  Unexpected version in status update, we need to start at 0
                    let data = self.expected_firmware;
                    let mtu = status.mtu.unwrap_or(128) as usize;
                    let to_copy = core::cmp::min(mtu, data.len());
                    let s = &data[..to_copy];
                    Ok(CommandRef::Write {
                        version: self.expected_version,
                        correlation_id: status.correlation_id,
                        offset: 0,
                        data: s,
                    })
                }
            } else {
                // No update status, start a new update
                let data = self.expected_firmware;
                let mtu = status.mtu.unwrap_or(128) as usize;
                let to_copy = core::cmp::min(mtu, data.len());
                let s = &data[..to_copy];
                Ok(CommandRef::Write {
                    version: self.expected_version,
                    correlation_id: status.correlation_id,
                    offset: 0,
                    data: s,
                })
            }
        }
    }
}
