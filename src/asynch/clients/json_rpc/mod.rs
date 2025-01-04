
use alloc::{string::ToString, vec};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{models::results::XRPLResponse, XRPLSerdeJsonError};

mod exceptions;
pub use exceptions::XRPLJsonRpcException;

use super::{client::XRPLClient, exceptions::XRPLClientResult};

/// Renames the requests field `command` to `method` for JSON-RPC.
fn request_to_json_rpc(request: &impl Serialize) -> XRPLClientResult<Value> {
    let mut json_rpc_request = Map::new();
    let request_value = serde_json::to_value(request)?;
    let mut request = request_value
        .as_object()
        .ok_or(XRPLSerdeJsonError::UnexpectedValueType {
            expected: "Object".to_string(),
            found: request_value.clone(),
        })?
        .clone();
    if let Some(command) = request.remove("command") {
        json_rpc_request.insert("method".to_string(), command);
        json_rpc_request.insert(
            "params".to_string(),
            serde_json::Value::Array(vec![Value::Object(request)]),
        );
    }

    Ok(Value::Object(json_rpc_request))
}

#[cfg(all(feature = "json-rpc", feature = "std"))]
mod _std {
    use crate::models::requests::XRPLRequest;
    #[cfg(feature = "helpers")]
    use crate::{asynch::clients::XRPLFaucet, models::requests::FundFaucet};
    #[cfg(feature = "helpers")]
    use alloc::string::ToString;

    use super::*;
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

    impl XRPLClient for AsyncJsonRpcClient {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            let client = HttpClient::new();
            let request_json_rpc = request_to_json_rpc(&request)?;
            let response = client
                .post(self.url.as_ref())
                .json(&request_json_rpc)
                .send()
                .await;
            match response {
                Ok(response) => match response.text().await {
                    Ok(response) => {
                        Ok(serde_json::from_str::<XRPLResponse<'b>>(&response).unwrap())
                    }
                    Err(error) => Err(error.into()),
                },
                Err(error) => Err(error.into()),
            }
        }

        fn get_host(&self) -> Url {
            self.url.clone()
        }
    }

    #[cfg(feature = "helpers")]
    impl XRPLFaucet for AsyncJsonRpcClient {
        async fn request_funding(
            &self,
            url: Option<Url>,
            request: FundFaucet<'_>,
        ) -> XRPLClientResult<()> {
            let faucet_url = self.get_faucet_url(url)?;
            let client = HttpClient::new();
            let request_json_rpc = serde_json::to_value(&request).unwrap();
            let response = client
                .post(faucet_url.to_string())
                .json(&request_json_rpc)
                .send()
                .await;
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(())
                    } else {
                        todo!()
                    }
                }
                Err(error) => Err(error.into()),
            }
        }
    }
}

#[cfg(all(feature = "json-rpc", not(feature = "std")))]
mod _no_std {
    use crate::{asynch::clients::SingleExecutorMutex, models::requests::XRPLRequest};
    #[cfg(feature = "helpers")]
    use crate::{asynch::clients::XRPLFaucet, models::requests::FundFaucet};

    use super::*;
    use alloc::sync::Arc;
    use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
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
        pub fn connect(url: Url, tcp: &'a T, dns: &'a D) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(HttpClient::new(tcp, dns))),
            }
        }

        pub fn connect_with_tls(url: Url, tcp: &'a T, dns: &'a D, tls: TlsConfig<'a>) -> Self {
            Self {
                url,
                client: Arc::new(Mutex::new(HttpClient::new_with_tls(tcp, dns, tls))),
            }
        }
    }

    impl<const BUF: usize, T, D, M> XRPLClient for AsyncJsonRpcClient<'_, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect,
        D: Dns,
    {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            let request_json_rpc = request_to_json_rpc(&request)?;
            let request_string = request_json_rpc.to_string();
            let request_buf = request_string.as_bytes();
            let mut rx_buffer = [0; BUF];
            let mut client = self.client.lock().await;
            let response = match client.request(Method::POST, self.url.as_str()).await {
                Ok(client) => {
                    if let Err(error) = client
                        .body(request_buf)
                        .content_type(ContentType::ApplicationJson)
                        .send(&mut rx_buffer)
                        .await
                    {
                        Err(error.into())
                    } else {
                        Ok(serde_json::from_slice::<XRPLResponse<'_>>(&rx_buffer)?)
                    }
                }
                Err(error) => Err(error.into()),
            };

            response
        }

        fn get_host(&self) -> Url {
            self.url.clone()
        }
    }

    #[cfg(feature = "helpers")]
    impl<'a, const BUF: usize, T, D, M> XRPLFaucet for AsyncJsonRpcClient<'a, BUF, T, D, M>
    where
        M: RawMutex,
        T: TcpConnect + 'a,
        D: Dns + 'a,
    {
        async fn request_funding(
            &self,
            url: Option<Url>,
            request: FundFaucet<'_>,
        ) -> XRPLClientResult<()> {
            let faucet_url = self.get_faucet_url(url)?;
            let request_json_rpc = serde_json::to_value(&request).unwrap();
            let request_string = request_json_rpc.to_string();
            let request_buf = request_string.as_bytes();
            let mut rx_buffer = [0; BUF];
            let mut client = self.client.lock().await;
            let response = match client.request(Method::POST, faucet_url.as_str()).await {
                Ok(client) => {
                    if let Err(error) = client
                        .body(request_buf)
                        .content_type(ContentType::ApplicationJson)
                        .send(&mut rx_buffer)
                        .await
                    {
                        Err(error.into())
                    } else {
                        let response = serde_json::from_slice::<XRPLResponse<'_>>(&rx_buffer)?;
                        if response.is_success() {
                            Ok(())
                        } else {
                            todo!()
                            // Err!(XRPLJsonRpcException::RequestError())
                        }
                    }
                }
                Err(error) => Err(XRPLJsonRpcException::ReqwlessError(error)),
            };

            response.map_err(Into::into)
        }
    }
}

#[cfg(all(feature = "json-rpc", not(feature = "std")))]
pub use _no_std::AsyncJsonRpcClient;
#[cfg(all(feature = "json-rpc", feature = "std"))]
pub use _std::AsyncJsonRpcClient;
