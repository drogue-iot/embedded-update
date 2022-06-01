use crate::traits::*;
use core::future::Future;
use drogue_ajour_protocol::{CommandRef, StatusRef};

/// Trait for the underlying transport (CoAP, HTTP, MQTT or LoRaWAN)
pub trait Transport {
    /// Error type
    type Error;

    /// Future returned by send
    type RequestFuture<'m>: Future<Output = Result<&'m mut [u8], Self::Error>> + 'm
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

impl<T> FirmwareUpdater<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    async fn report<'m>(
        &mut self,
        status: StatusRef<'m>,
        rx: &'m mut [u8],
    ) -> Result<CommandRef<'m>, Error> {
        let payload = serde_cbor::to_vec(&status).map_err(|_| Error::Encode)?;
        let result = self.transport.request(&payload, rx).await;
        match result {
            Ok(payload) => {
                if let Ok(cmd) = serde_cbor::from_slice::<CommandRef>(&payload) {
                    Ok(cmd)
                } else {
                    Err(Error::Decode)
                }
            }
            Err(_) => Err(Error::Transport),
        }
    }

    async fn check<F: FirmwareDevice>(
        &mut self,
        initial: FirmwareStatus<'_>,
        device: &mut F,
    ) -> Result<bool, Error> {
        let mut status = if let Some(next) = &initial.next_version {
            StatusRef::update(
                &initial.current_version,
                Some(F::MTU),
                initial.offset,
                initial.next_version,
                None,
            )
        } else {
            StatusRef::first(&initial.current_version, Some(F::MTU), None)
        };
        let mut rx_buf = [0; F::MTU + 12];

        #[allow(unused_mut)]
        #[allow(unused_assignments)]
        loop {
            let cmd = self.report(status).await?;
            match cmd {
                Command::Write {
                    version,
                    offset,
                    data,
                    correlation_id: _,
                } => {
                    v = version.clone();
                    if offset == 0 {
                        println!(
                            "Updating device firmware from {} to {}",
                            current_version, version
                        );
                        device.start(&v).await?;
                    }
                    device.write(offset, &data).await?;
                    status = StatusRef::update(
                        &current_version,
                        Some(F::MTU),
                        offset + data.len() as u32,
                        &v,
                        None,
                    );
                }
                Command::Sync {
                    version: _,
                    poll: _,
                    correlation_id: _,
                } => {
                    log::info!("Firmware in sync");
                    device.synced().await?;
                    return Ok(true);
                }
                Command::Wait {
                    poll,
                    correlation_id: _,
                } => {
                    if let Some(poll) = poll {
                        println!("Instructed to wait {} seconds", poll);
                        sleep(Duration::from_secs(poll as u64)).await;
                    }
                }
                Command::Swap {
                    version,
                    checksum,
                    correlation_id: _,
                } => {
                    println!("Firmware written, instructing device to swap");
                    device.swap(&version, checksum).await?;
                    return Ok(false);
                }
            }
        }
    }

    /// Run the firmware update protocol. Returns when firmware is fully in sync
    pub async fn run<F: FirmwareDevice>(&self, device: &mut F) -> Result<bool, Error> {
        let initial = device.status().await.map_err(|_| Error::Device)?;
        if self.check(intitial, device).await? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
