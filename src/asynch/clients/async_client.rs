use super::client::Client;
use crate::models::{requests::Request, results::XRPLResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(async_fn_in_trait)]
pub trait AsyncClient<'a>: Client<'a> {
    async fn request<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &'a self,
        request: Req,
    ) -> Result<XRPLResponse<'_, Res, Req>> {
        self.request_impl(request).await
    }
}

impl<'a, T: Client<'a>> AsyncClient<'a> for T {}
