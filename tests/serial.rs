#![cfg_attr(feature = "nightly", feature(generic_associated_types))]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

use core::future::Future;
use embedded_update::{device, service, FirmwareUpdater};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_serial_chain() {
    let mut t1 = Timer;
    let mut t2 = Timer;
    let (src, dest) = Link::new();
    let firmware = service::InMemory::new(b"2", &[1; 1024]);
    let mut updater_1 = FirmwareUpdater::new(firmware, Default::default());

    let mut serial_device = device::Serial::new(src);

    let u1_fut = updater_1.run(&mut serial_device, &mut t1);

    let serial_service = service::Serial::new(dest);

    let mut updater_2 = FirmwareUpdater::new(serial_service, Default::default());
    let mut device = device::Simulator::new(b"1");

    let u2_fut = updater_2.run(&mut device, &mut t2);

    let (r1, r2) = tokio::join!(u1_fut, u2_fut);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert_eq!(device.version(), b"2");
}

type Frame = [u8; 1024];

struct Link {
    tx: mpsc::Sender<Frame>,
    rx: mpsc::Receiver<Frame>,
}

impl Link {
    pub fn new() -> (Link, Link) {
        let (src_tx, src_rx) = mpsc::channel(4);
        let (dest_tx, dest_rx) = mpsc::channel(4);
        let src = Link {
            tx: src_tx,
            rx: dest_rx,
        };

        let dest = Link {
            tx: dest_tx,
            rx: src_rx,
        };
        (src, dest)
    }
}

impl embedded_io::Io for Link {
    type Error = std::io::Error;
}

impl embedded_io::asynch::Read for Link {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            if let Some(m) = self.rx.recv().await {
                let to_copy = core::cmp::min(m.len(), buf.len());
                buf[..to_copy].copy_from_slice(&m[..to_copy]);
                Ok(to_copy)
            } else {
                Ok(0)
            }
        }
    }
}

impl embedded_io::asynch::Write for Link {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            for chunk in buf.chunks(1024) {
                let mut b = [0; 1024];
                b[..chunk.len()].copy_from_slice(chunk);
                self.tx.send(b).await.unwrap();
            }
            Ok(buf.len())
        }
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        async move { todo!() }
    }
}

pub struct Timer;

impl embedded_hal_async::delay::DelayUs for Timer {
    type Error = core::convert::Infallible;
    type DelayUsFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm where Self: 'm;
    fn delay_us(&mut self, i: u32) -> Self::DelayUsFuture<'_> {
        async move {
            tokio::time::sleep(tokio::time::Duration::from_micros(i as u64)).await;
            Ok(())
        }
    }

    type DelayMsFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm where Self: 'm;
    fn delay_ms(&mut self, i: u32) -> Self::DelayMsFuture<'_> {
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(i as u64)).await;
            Ok(())
        }
    }
}
