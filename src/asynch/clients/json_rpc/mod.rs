use alloc::{string::String, sync::Arc};
use anyhow::Result;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use serde::{Deserialize, Serialize};

use crate::{
    models::{requests::Request, results::XRPLResponse},
    Err,
};

mod exceptions;
pub use exceptions::XRPLJsonRpcException;

use super::{client::Client, SingleExecutorMutex};

/// Renames the requests field `command` to `method` for JSON-RPC.
fn request_to_json_rpc(request: &impl Serialize) -> Result<String> {
    let mut request = match serde_json::to_value(request) {
        Ok(request) => request,
        Err(error) => return Err!(error),
    };
    if let Some(command) = request.get_mut("command") {
        let method = command.take();
        request["method"] = method;
    }
    match serde_json::to_string(&request) {
        Ok(request) => Ok(request),
        Err(error) => Err!(error),
    }
}

#[cfg(feature = "json-rpc-std")]
mod std_client {
    use super::*;
    use reqwest::Client as HttpClient;
    use url::Url;

    pub struct AsyncJsonRpcClient<M = SingleExecutorMutex>
    where
        M: RawMutex,
    {
        url: Url,
        client: Arc<Mutex<M, HttpClient>>,
    }

    impl<M> AsyncJsonRpcClient<M>
    where
        M: RawMutex,
    {
        pub fn new(url: Url) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(HttpClient::new())),
            }
        }
    }

    impl<M> AsyncJsonRpcClient<M>
    where
        M: RawMutex,
    {
        fn from(url: Url, client: HttpClient) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(client)),
            }
        }
    }

    impl<M> Client for AsyncJsonRpcClient<M>
    where
        M: RawMutex,
    {
        async fn request_impl<
            'a: 'b,
            'b,
            Res: Serialize + for<'de> Deserialize<'de>,
            Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
        >(
            &self,
            request: Req,
        ) -> Result<XRPLResponse<'b, Res, Req>> {
            let client = self.client.lock().await;
            match client
                .post(self.url.as_ref())
                .body(request_to_json_rpc(&request)?)
                .send()
                .await
            {
                Ok(response) => match response.json().await {
                    Ok(response) => Ok(response),
                    Err(error) => Err!(error),
                },
                Err(error) => Err!(error),
            }
        }
    }
}

#[cfg(feature = "json-rpc")]
mod no_std_client {
    use super::*;
    use embedded_nal_async::{Dns, TcpConnect};
    use reqwless::{
        client::{HttpClient, TlsConfig},
        headers::ContentType,
        request::{Method, RequestBuilder},
    };
    use url::Url;

    pub struct AsyncJsonRpcClient<'a, const BUF: usize, T, D, M = SingleExecutorMutex>
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a,
    {
        url: Url,
        client: Arc<Mutex<M, HttpClient<'a, T, D>>>,
    }

    impl<'a, const BUF: usize, T, D, M> AsyncJsonRpcClient<'a, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a,
    {
        pub fn new(url: Url, tcp: &'a T, dns: &'a D) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(HttpClient::new(tcp, dns))),
            }
        }

        pub fn new_with_tls(url: Url, tcp: &'a T, dns: &'a D, tls: TlsConfig<'a>) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(HttpClient::new_with_tls(tcp, dns, tls))),
            }
        }
    }

    impl<const BUF: usize, T, D, M> Client for AsyncJsonRpcClient<'_, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect,
        D: Dns,
    {
        async fn request_impl<
            'a: 'b,
            'b,
            Res: Serialize + for<'de> Deserialize<'de>,
            Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
        >(
            &self,
            request: Req,
        ) -> Result<XRPLResponse<'b, Res, Req>> {
            let request_json_rpc = request_to_json_rpc(&request)?;
            let request_buf = request_json_rpc.as_bytes();
            let mut rx_buffer = [0; BUF];
            let mut client = self.client.lock().await;
            let response = match client.request(Method::POST, self.url.as_str()).await {
                Ok(client) => {
                    if let Err(_error) = client
                        .body(request_buf)
                        .content_type(ContentType::TextPlain)
                        .send(&mut rx_buffer)
                        .await
                    {
                        Err!(XRPLJsonRpcException::ReqwlessError)
                    } else {
                        match serde_json::from_slice::<XRPLResponse<'_, Res, Req>>(&rx_buffer) {
                            Ok(response) => Ok(response),
                            Err(error) => Err!(error),
                        }
                    }
                }
                Err(_error) => Err!(XRPLJsonRpcException::ReqwlessError),
            };

            response
        }
    }
}

#[cfg(all(feature = "json-rpc", not(feature = "json-rpc-std")))]
pub use no_std_client::AsyncJsonRpcClient;
#[cfg(all(feature = "json-rpc-std", not(feature = "json-rpc")))]
pub use std_client::AsyncJsonRpcClient;
