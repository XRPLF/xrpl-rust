use super::client::Client;
use crate::models::{requests::Request, results::XRPLResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(async_fn_in_trait)]
pub trait AsyncClient: Client {
    async fn request<
        'a: 'b,
        'b,
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &self,
        request: Req,
    ) -> Result<XRPLResponse<'b, Res, Req>> {
        self.request_impl(request).await
    }
}

impl<T: Client> AsyncClient for T {}
