use super::client::Client;
use crate::models::{requests::Request, results::XRPLResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait AsyncClient: Client {
    async fn request<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + for<'a> Request<'a>,
    >(
        &self,
        request: Req,
    ) -> Result<XRPLResponse<'_, Res, Req>> {
        self.request_impl(request).await
    }
}

impl<T: Client> AsyncClient for T {}
