use crate::{Command, Status, UpdateService};
use core::future::Future;
use embedded_nal_async::{SocketAddr, TcpClient};
use heapless::String;
use rand_core::{CryptoRng, RngCore};
use reqwless::{
    client::{Error as HttpError, HttpClient},
    request::{ContentType, Request, Status as ResponseStatus},
};
use serde::Serialize;

#[cfg(feature = "tls")]
use embedded_tls::*;

/// An update service implementation for the Drogue Cloud update service.
pub struct Hawkbit<'a, T, RNG, const MTU: usize>
where
    T: TcpClient + 'a,
    RNG: RngCore + CryptoRng + 'a,
{
    client: T,
    rng: RNG,
    addr: SocketAddr,
    host: &'a str,
    path: &'a str,
    token: &'a str,
    buf: [u8; MTU],
    registered: bool,
}

impl<'a, T, RNG, const MTU: usize> Hawkbit<'a, T, RNG, MTU>
where
    T: TcpClient + 'a,
    RNG: RngCore + CryptoRng + 'a,
{
    /// Construct a new Drogue update service
    pub fn new(client: T, rng: RNG, addr: SocketAddr, host: &'a str, path: &'a str, token: &'a str) -> Self {
        Self {
            client,
            rng,
            addr,
            host,
            path,
            token,
            buf: [0; MTU],
            registered: false,
        }
    }
}

/// An error returned from the update service.
#[derive(Debug)]
pub enum Error<N, H, S, T> {
    /// Error from the underlying network
    Network(N),
    /// Error from HTTP client
    Http(H),
    /// Error from TLS
    Tls(T),
    /// Error in encoding or decoding of the payload
    Codec(S),
    /// Error in the firmware update protocol
    Protocol,
}

#[derive(Serialize, Deserialize)]
pub struct ControllerAttributes {
    mode: ControllerMode,
}

#[derive(Serialize, Deserialize)]
pub enum ControllerMode {
    Merge
}

impl<'a, T, RNG, const MTU: usize> UpdateService for Hawkbit<'a, T, RNG, MTU>
where
    T: TcpClient + 'a,
    RNG: RngCore + CryptoRng + 'a,
{
    #[cfg(feature = "tls")]
    type Error = Error<T::Error, HttpError, serde_json_core::Error, TlsError>;

    #[cfg(not(feature = "tls"))]
    type Error = Error<T::Error, HttpError, serde_json_core::Error, ()>;

    type RequestFuture<'m> = impl Future<Output = Result<Command<'m>, Self::Error>> + 'm where Self: 'm;
    fn request<'m>(&'m mut self, status: &'m Status<'m>) -> Self::RequestFuture<'m> {
        async move {
            #[allow(unused_mut)]
            let mut connection = self.client.connect(self.addr).await.map_err(Error::Network)?;

            #[cfg(feature = "tls")]
            let mut tls_buffer = [0; 6000];

            #[cfg(feature = "tls")]
            let mut connection = {
                let mut connection: TlsConnection<'_, _, Aes128GcmSha256> =
                    TlsConnection::new(connection, &mut tls_buffer);
                connection
                    .open::<_, NoClock, 1>(TlsContext::new(
                        &TlsConfig::new().with_server_name(self.host),
                        &mut self.rng,
                    ))
                    .await
                    .map_err(Error::Tls)?;
                connection
            };
            let mut client = HttpClient::new(&mut connection, self.host);

            // Register first time invoked
            if !registered {
                // TODO: Get attributes from somewhere
                let attributes = "{
                  "mode": "merge",
                  "data": {
                    "VIN": "JH4TB2H26CC000001",
                    "hwRevision": "1"
                  },
                  "status": {
                    "result": {
                      "finished": "success"
                    },
                    "execution": "closed",
                    "details": []
                  }
                }};

                let mut auth = String::new();
                write!(auth, "GatewayToken {}", self.token).map_err(|_| Error::Protocol)?;

                let request = Request::put()
                    .path(self.path)
                    .payload(&attributes[..])
                    .content_type(ContentType::ApplicationJson)
                    .headers(&[("Authorization", auth.as_str()), ("Accept", "application/hal+json")])
                    .build();

                let mut rx_buf = [0; MTU];
                let res = client.request(request, &mut rx_buf).await.map_err(Error::Http);
                match res {
                    Ok(_) => {
                        debug!("Successfully set attributes");
                    }
                    Err(e) => {
                        warn!("Error setting attributes: {:?}", e);
                    }
                }
                self.registered = true;
            }

            todo!()
            /*
                        let mut payload = [0; 64];
                        let writer = serde_cbor::ser::SliceWrite::new(&mut payload[..]);
                        let mut ser = serde_cbor::Serializer::new(writer).packed_format();
                        status.serialize(&mut ser).map_err(Error::Codec)?;
                        let writer = ser.into_inner();
                        let size = writer.bytes_written();
                        debug!("Status payload is {} bytes", size);

                        let request = Request::post()
                            .path("/v1/dfu?ct=30")
                            .payload(&payload[..size])
                            .basic_auth(self.username, self.password)
                            .content_type(ContentType::ApplicationCbor)
                            .build();

                        let response = client.request(request, &mut rx_buf).await.map_err(Error::Http)?;

                        if response.status == ResponseStatus::Ok
                            || response.status == ResponseStatus::Accepted
                            || response.status == ResponseStatus::Created
                        {
                            if let Some(payload) = response.payload {
                                self.buf[..payload.len()].copy_from_slice(payload);
                                let command: Command<'m> =
                                    serde_cbor::de::from_mut_slice(&mut self.buf[..payload.len()]).map_err(Error::Codec)?;
                                Ok(command)
                            } else {
                                Ok(Command::new_wait(Some(10), None))
                            }
                        } else {
                            Err(Error::Protocol)
                        }
            */
        }
    }
}
