use alloc::{borrow::Cow, sync::Arc};
use anyhow::Result;
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};
use embedded_nal_async::{Dns, TcpConnect};
use reqwless::{
    client::HttpClient,
    headers::ContentType,
    request::{Method, RequestBuilder},
};
use serde::{Deserialize, Serialize};

use crate::{
    models::{requests::Request, results::XRPLResponse},
    Err,
};

mod exceptions;
pub use exceptions::XRPLJsonRpcException;

use super::{client::Client, SingleExecutorMutex};

pub struct AsyncJsonRpcClient<'a, const BUF: usize, T, D, M = SingleExecutorMutex>
where
    M: RawMutex,
    T: TcpConnect + 'a,
    D: Dns + 'a,
{
    url: Cow<'a, str>,
    client: Arc<Mutex<M, HttpClient<'a, T, D>>>,
}

impl<'a, const BUF: usize, T, D, M> AsyncJsonRpcClient<'a, BUF, T, D, M>
where
    M: RawMutex,
    T: TcpConnect + 'a,
    D: Dns + 'a,
{
    pub fn new(url: Cow<'a, str>, tcp: &'a T, dns: &'a D) -> Self {
        Self {
            url,
            client: Arc::new(Mutex::new(HttpClient::new(tcp, dns))),
        }
    }
}

impl<'a, const BUF: usize, T, D, M> Client<'a> for AsyncJsonRpcClient<'a, BUF, T, D, M>
where
    M: RawMutex,
    T: TcpConnect + 'a,
    D: Dns + 'a,
{
    async fn request_impl<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &'a self,
        request: Req,
    ) -> Result<XRPLResponse<'_, Res, Req>> {
        let request_buf = match serde_json::to_vec(&request) {
            Ok(request) => request,
            Err(error) => return Err!(error),
        };
        let mut rx_buffer = [0; BUF];
        let mut client = self.client.lock().await;
        let response = match client.request(Method::POST, &self.url).await {
            Ok(client) => {
                if let Err(error) = client
                    .body(request_buf.as_slice())
                    .content_type(ContentType::TextPlain)
                    .send(&mut rx_buffer)
                    .await
                {
                    Err!(XRPLJsonRpcException::ReqwlessError(error))
                } else {
                    match serde_json::from_slice::<XRPLResponse<'_, Res, Req>>(&rx_buffer) {
                        Ok(response) => Ok(response),
                        Err(error) => Err!(error),
                    }
                }
            }
            Err(error) => Err!(XRPLJsonRpcException::ReqwlessError(error)),
        };

        response
    }
}
