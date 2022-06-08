use crate::protocol::{Command, Status};
use crate::traits::{FirmwareDevice, FirmwareVersion, UpdateService};
use embedded_hal_async::delay::DelayUs;
use futures::{
    future::{select, Either},
    pin_mut,
};

/// The error types that the updater may return during the update process.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<D, S> {
    Encode,
    Decode,
    Delay,
    Device(D),
    Service(S),
}

/// The device status as determined after running the updater.
#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceStatus {
    Synced,
    Updated,
}

#[derive(Clone)]
struct UpdaterState<F>
where
    F: FirmwareVersion,
{
    current_version: F,
    next_offset: u32,
    next_version: Option<F>,
}

/// Configuration for the updater task.
pub struct UpdaterConfig {
    /// Timeout used for update requests in milliseconds.
    pub timeout_ms: u32,
    /// Backoff time when updates fail or time out.
    pub backoff_ms: u32,
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 15_000,
            backoff_ms: 1_000,
        }
    }
}

/// The updater process that uses the update service to perform a firmware update check
/// for a device. If the device needs to be updated, the updater will follow the update protocol
pub struct FirmwareUpdater<T>
where
    T: UpdateService,
{
    service: T,
    timeout_ms: u32,
    backoff_ms: u32,
}

impl<T> FirmwareUpdater<T>
where
    T: UpdateService,
{
    /// Create a new instance of the updater with the provided service instance.
    pub fn new(service: T, config: UpdaterConfig) -> Self {
        Self {
            service,
            timeout_ms: config.timeout_ms,
            backoff_ms: config.backoff_ms,
        }
    }

    async fn check<F: FirmwareDevice, D: DelayUs>(
        &mut self,
        device: &mut F,
        delay: &mut D,
    ) -> Result<bool, Error<F::Error, T::Error>> {
        let mut state = {
            let initial = device.status().await.map_err(|e| Error::Device(e))?;
            UpdaterState {
                current_version: initial.current_version,
                next_offset: initial.next_offset,
                next_version: initial.next_version,
            }
        };

        #[allow(unused_mut)]
        #[allow(unused_assignments)]
        #[allow(renamed_and_removed_lints)]
        #[allow(mutable_borrow_reservation_conflict)]
        loop {
            let status = if let Some(next) = &state.next_version {
                Status::update(
                    state.current_version.as_ref(),
                    Some(F::MTU as u32),
                    state.next_offset,
                    next.as_ref(),
                    None,
                )
            } else {
                Status::first(state.current_version.as_ref(), Some(F::MTU as u32), None)
            };

            debug!("Sending status: {:?}", status);

            let mut next_state = state.clone();
            let mut poll_opt = Some(self.backoff_ms / 1000);
            {
                let delay_fut = delay.delay_ms(self.timeout_ms);
                let cmd_fut = self.service.request(&status);
                pin_mut!(delay_fut);
                pin_mut!(cmd_fut);
                match select(delay_fut, cmd_fut).await {
                    Either::Right((cmd, _)) => match cmd {
                        Ok(Command::Write {
                            version,
                            offset,
                            data,
                            correlation_id: _,
                        }) => {
                            if offset == 0 {
                                debug!(
                                    "Updating device firmware from {} to {}",
                                    state.current_version,
                                    version.as_ref()
                                );
                                device
                                    .start(version.as_ref())
                                    .await
                                    .map_err(|e| Error::Device(e))?;
                            }
                            device
                                .write(offset, data.as_ref())
                                .await
                                .map_err(|e| Error::Device(e))?;

                            next_state.next_offset += data.len() as u32;
                            next_state.next_version.replace(
                                F::Version::from_slice(version.as_ref())
                                    .map_err(|_| Error::Decode)?,
                            );
                        }
                        Ok(Command::Sync {
                            version: _,
                            poll,
                            correlation_id: _,
                        }) => {
                            debug!("Device firmware is up to date");
                            device.synced().await.map_err(|e| Error::Device(e))?;
                            poll_opt = poll;
                            return Ok(true);
                        }
                        Ok(Command::Wait {
                            poll,
                            correlation_id: _,
                        }) => {
                            debug!("Instruction to wait for {:?} seconds", poll);
                            poll_opt = poll;
                        }
                        Ok(Command::Swap {
                            version,
                            checksum,
                            correlation_id: _,
                        }) => {
                            debug!("Swaping firmware");
                            device
                                .update(version.as_ref(), checksum.as_ref())
                                .await
                                .map_err(|e| Error::Device(e))?;
                            return Ok(false);
                        }
                        Err(e) => {
                            #[cfg(feature = "defmt")]
                            debug!("Error reporting status: {:?}", defmt::Debug2Format(&e));
                            #[cfg(not(feature = "defmt"))]
                            debug!("Error reporting status: {:?}", e);
                        }
                    },
                    _ => {}
                }
            }
            state = next_state;
            if let Some(poll) = poll_opt {
                delay
                    .delay_ms(poll * 1000)
                    .await
                    .map_err(|_| Error::Delay)?;
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
    ) -> Result<DeviceStatus, Error<F::Error, T::Error>> {
        if self.check(device, delay).await? {
            Ok(DeviceStatus::Synced)
        } else {
            Ok(DeviceStatus::Updated)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use core::future::Future;

    use crate::DeviceStatus;
    use crate::FirmwareUpdater;
    use crate::InMemory;
    use crate::Simulator;
    use crate::UpdaterConfig;

    pub struct TokioDelay;

    impl embedded_hal_async::delay::DelayUs for TokioDelay {
        type Error = Infallible;

        type DelayUsFuture<'a> = impl Future<Output = Result<(), Self::Error>>
        where
            Self: 'a;

        fn delay_us(&mut self, us: u32) -> Self::DelayUsFuture<'_> {
            async move {
                tokio::time::sleep(tokio::time::Duration::from_micros(us as u64)).await;
                Ok(())
            }
        }

        type DelayMsFuture<'a> = impl Future<Output = Result<(), Self::Error>>
        where
            Self: 'a;

        fn delay_ms(&mut self, ms: u32) -> Self::DelayMsFuture<'_> {
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(ms as u64)).await;
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_update_protocol_synced() {
        let service = InMemory::new(b"1", &[1; 1024]);
        let mut device = Simulator::new(b"1");

        let mut updater = FirmwareUpdater::new(
            service,
            UpdaterConfig {
                timeout_ms: 1_000,
                backoff_ms: 0,
            },
        );
        let status = updater.run(&mut device, &mut TokioDelay).await.unwrap();
        assert_eq!(status, DeviceStatus::Synced);
    }

    #[tokio::test]
    async fn test_update_protocol_updated() {
        let service = InMemory::new(b"2", &[1; 1024]);
        let mut device = Simulator::new(b"1");

        let mut updater = FirmwareUpdater::new(
            service,
            UpdaterConfig {
                timeout_ms: 1_000,
                backoff_ms: 0,
            },
        );
        let status = updater.run(&mut device, &mut TokioDelay).await.unwrap();
        assert_eq!(status, DeviceStatus::Updated);
    }
}
