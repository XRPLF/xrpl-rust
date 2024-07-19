use super::client::Client;
use crate::models::{requests::XRPLRequest, results::XRPLResponse};
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait AsyncClient: Client {
    async fn request<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>> {
        self.request_impl(request).await
    }
}

impl<T: Client> AsyncClient for T {}
