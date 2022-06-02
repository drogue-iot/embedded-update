use crate::traits::FirmwareDevice;
use core::future::Future;
use drogue_ajour_protocol::{CommandRef, StatusRef};
use embedded_hal_async::delay::DelayUs;
use heapless::Vec;

/// Trait for the underlying transport (CoAP, HTTP, MQTT or LoRaWAN)
pub trait Transport {
    /// Error type
    type Error;

    /// Future returned by send
    type RequestFuture<'m>: Future<Output = Result<usize, Self::Error>> + 'm
    where
        Self: 'm;
    /// Send payload to server.
    fn request<'m>(&'m mut self, tx: &'m [u8], rx: &'m mut [u8]) -> Self::RequestFuture<'m>;
}

pub struct FirmwareUpdater<T>
where
    T: Transport,
{
    transport: T,
}

pub enum Error {
    Encode,
    Decode,
    Device,
    Transport,
}

pub enum DeviceStatus {
    Synced,
    Updated,
}

const MAX_OVERHEAD: usize = 42;

struct UpdaterState {
    current_version: Vec<u8, 32>,
    next_offset: u32,
    next_version: Option<Vec<u8, 32>>,
}

impl<T> FirmwareUpdater<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    async fn report<'m>(
        &mut self,
        status: &StatusRef<'_>,
        rx: &'m mut [u8],
    ) -> Result<CommandRef<'m>, Error> {
        let payload = serde_cbor::to_vec(&status).map_err(|_| Error::Encode)?;
        let result = self.transport.request(&payload, rx).await;
        match result {
            Ok(len) => {
                if let Ok(cmd) = serde_cbor::from_slice::<CommandRef>(&rx[..len]) {
                    Ok(cmd)
                } else {
                    Err(Error::Decode)
                }
            }
            Err(_) => Err(Error::Transport),
        }
    }

    async fn check<F: FirmwareDevice, D: DelayUs>(
        &mut self,
        device: &mut F,
        delay: &mut D,
    ) -> Result<bool, Error>
    where
        [(); F::MTU + MAX_OVERHEAD]:,
    {
        let mut state = {
            let initial = device.status().await.map_err(|_| Error::Device)?;
            UpdaterState {
                current_version: Vec::from_slice(initial.current_version)
                    .map_err(|_| Error::Encode)?,
                next_offset: initial.next_offset,
                next_version: if let Some(next_version) = &initial.next_version {
                    Some(Vec::from_slice(next_version).map_err(|_| Error::Encode)?)
                } else {
                    None
                },
            }
        };

        let mut rx_buf = [0; { F::MTU + MAX_OVERHEAD }];

        #[allow(unused_mut)]
        #[allow(unused_assignments)]
        loop {
            let status = if let Some(next) = &state.next_version {
                StatusRef::update(
                    &state.current_version,
                    Some(F::MTU as u32),
                    state.next_offset,
                    next,
                    None,
                )
            } else {
                StatusRef::first(&state.current_version, Some(F::MTU as u32), None)
            };

            let cmd = self.report(&status, &mut rx_buf).await?;
            match cmd {
                CommandRef::Write {
                    version,
                    offset,
                    data,
                    correlation_id: _,
                } => {
                    if offset == 0 {
                        debug!(
                            "Updating device firmware from {} to {}",
                            state.current_version, version
                        );
                        device.start(version).await.map_err(|_| Error::Device)?;
                    }
                    device
                        .write(offset, &data)
                        .await
                        .map_err(|_| Error::Device)?;
                    state.next_offset += data.len() as u32;
                    state
                        .next_version
                        .replace(Vec::from_slice(version).map_err(|_| Error::Decode)?);
                }
                CommandRef::Sync {
                    version: _,
                    poll: _,
                    correlation_id: _,
                } => {
                    debug!("Device firmware is up to date");
                    device.synced().await.map_err(|_| Error::Device)?;
                    return Ok(true);
                }
                CommandRef::Wait {
                    poll,
                    correlation_id: _,
                } => {
                    debug!("Instruction to wait for {:?} seconds", poll);
                    if let Some(poll) = poll {
                        delay
                            .delay_ms(poll * 1000)
                            .await
                            .map_err(|_| Error::Device)?;
                    }
                }
                CommandRef::Swap {
                    version,
                    checksum,
                    correlation_id: _,
                } => {
                    debug!("Swaping firmware");
                    device
                        .update(&version, checksum)
                        .await
                        .map_err(|_| Error::Device)?;
                    return Ok(false);
                }
            }
        }
    }

    /// Run the firmware update protocol. The update is finished with two outcomes:
    ///
    /// 1) The device is in sync, in which case `DeviceStatus::Synced` is returned.
    /// 2) The device is updated, in which case `DeviceStatus::Updated` is returned. It is the responsibility
    ///    of called to reset the device in order to run the new firmware.
    pub async fn run<F: FirmwareDevice, D: DelayUs>(
        &mut self,
        device: &mut F,
        delay: &mut D,
    ) -> Result<DeviceStatus, Error>
    where
        [(); F::MTU + MAX_OVERHEAD]:,
    {
        if self.check(device, delay).await? {
            Ok(DeviceStatus::Synced)
        } else {
            // Reset device
            Ok(DeviceStatus::Updated)
        }
    }
}
