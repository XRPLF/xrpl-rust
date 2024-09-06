use alloc::{
    dbg,
    string::{String, ToString},
    sync::Arc,
    vec,
};
use anyhow::Result;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use serde::Serialize;
use serde_json::{Map, Value};
use url::Url;

use crate::{
    asynch::wallet::get_faucet_url,
    models::{requests::FundFaucet, results::XRPLResponse},
    Err,
};

mod exceptions;
pub use exceptions::XRPLJsonRpcException;

use super::client::Client;

pub trait XRPLFaucet: Client {
    fn get_faucet_url(&self, url: Option<Url>) -> Result<Url>
    where
        Self: Sized + Client,
    {
        get_faucet_url(self, url)
    }

    async fn request_funding(&self, url: Option<Url>, request: FundFaucet<'_>) -> Result<()>;
}

/// Renames the requests field `command` to `method` for JSON-RPC.
fn request_to_json_rpc(request: &impl Serialize) -> Result<Value> {
    let mut json_rpc_request = Map::new();
    let mut request = match serde_json::to_value(request) {
        Ok(request) => match request.as_object().cloned() {
            Some(request) => request,
            None => todo!("Handle non-object requests"),
        },
        Err(error) => return Err!(error),
    };
    if let Some(command) = request.remove("command") {
        json_rpc_request.insert("method".to_string(), command);
        json_rpc_request.insert(
            "params".to_string(),
            serde_json::Value::Array(vec![Value::Object(request)]),
        );
    }

    Ok(Value::Object(json_rpc_request))
}

#[cfg(feature = "json-rpc-std")]
mod _std {
    use crate::models::requests::{FundFaucet, XRPLRequest};

    use super::*;
    use alloc::{dbg, format, string::ToString};
    use reqwest::Client as HttpClient;
    use url::Url;

    pub struct AsyncJsonRpcClient {
        url: Url,
    }

    impl AsyncJsonRpcClient {
        pub fn connect(url: Url) -> Self {
            Self { url }
        }
    }

    impl Client for AsyncJsonRpcClient {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> Result<XRPLResponse<'b>> {
            let client = HttpClient::new();
            let request_json_rpc = request_to_json_rpc(&request)?;
            dbg!(&request_json_rpc);
            let response = client
                .post(self.url.as_ref())
                .json(&request_json_rpc)
                .send()
                .await;
            dbg!(&response);
            match response {
                Ok(response) => match response.text().await {
                    Ok(response) => {
                        dbg!(&response);
                        Ok(serde_json::from_str::<XRPLResponse<'b>>(&response).unwrap())
                    }
                    Err(error) => Err!(error),
                },
                Err(error) => Err!(error),
            }
        }

        fn get_host(&self) -> Url {
            self.url.clone()
        }
    }

    impl XRPLFaucet for AsyncJsonRpcClient {
        async fn request_funding(&self, url: Option<Url>, request: FundFaucet<'_>) -> Result<()> {
            let faucet_url = self.get_faucet_url(url)?;
            let client = HttpClient::new();
            let request_json_rpc = serde_json::to_value(&request).unwrap();
            dbg!(&request_json_rpc);
            let response = client
                .post(&faucet_url.to_string())
                .json(&request_json_rpc)
                .send()
                .await;
            dbg!(&response);
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        dbg!("Success");
                        dbg!(&response);
                        Ok(())
                    } else {
                        dbg!("Error");
                        dbg!(&response);
                        todo!()
                        // Err!(XRPLJsonRpcException::RequestError())
                    }
                }
                Err(error) => {
                    dbg!("req Error");
                    Err!(error)
                }
            }
        }
    }
}

#[cfg(feature = "json-rpc")]
mod _no_std {
    use crate::models::requests::XRPLRequest;

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
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> Result<XRPLResponse<'b>> {
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
                        match serde_json::from_slice::<XRPLResponse<'_>>(&rx_buffer) {
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
pub use _no_std::AsyncJsonRpcClient;
#[cfg(all(feature = "json-rpc-std", not(feature = "json-rpc")))]
pub use _std::AsyncJsonRpcClient;
