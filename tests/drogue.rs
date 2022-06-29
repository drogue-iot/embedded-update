#![cfg_attr(feature = "nightly", feature(generic_associated_types))]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

use core::future::Future;
use embedded_io::adapters::FromTokio;
use embedded_nal_async::{IpAddr, Ipv4Addr, SocketAddr};
use embedded_update::{device, service, FirmwareUpdater};
use rand::rngs::OsRng;
use std::env;
use tokio::net::TcpStream;

#[tokio::test]
async fn test_drogue_update() {
    env_logger::init();
    let username = env::var("DROGUE_CLOUD_USER");
    let password = env::var("DROGUE_CLOUD_PASSWORD");
    match (username, password) {
        (Ok(username), Ok(password)) => {
            let host = "http.sandbox.drogue.cloud";
            let port = 443;
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(65, 108, 135, 161)), port);
            let firmware: service::DrogueHttp<'_, _, _, 512> =
                service::DrogueHttp::new(TokioTcpClient, OsRng, addr, host, &username, &password);
            let mut updater = FirmwareUpdater::new(firmware, Default::default());
            let mut device = device::Simulator::new(b"1");

            let _ = updater.run(&mut device, &mut Timer).await;
            assert_eq!(device.version(), b"145024b");
        }
        _ => {
            if let Ok(_) = env::var("CI") {
                assert!(false, "Missing credentials");
            } else {
                println!("Skipping drogue tests");
            }
        }
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

pub struct TokioTcpClient;

impl embedded_io::Io for TokioTcpClient {
    type Error = std::io::Error;
}

impl embedded_nal_async::TcpClient for TokioTcpClient {
    type TcpConnection<'m> = FromTokio<TcpStream>;
    type ConnectFuture<'m> = impl Future<Output = Result<Self::TcpConnection<'m>, Self::Error>> + 'm
    where
        Self: 'm;
    fn connect<'m>(&'m mut self, remote: embedded_nal_async::SocketAddr) -> Self::ConnectFuture<'m> {
        async move {
            match TcpStream::connect(format!("{}:{}", remote.ip(), remote.port())).await {
                Ok(stream) => Ok(FromTokio::new(stream)),
                Err(e) => Err(e),
            }
        }
    }
}
