use super::{client::XRPLClient, CommonFields};
use crate::models::{
    requests::{server_state::ServerState, XRPLRequest},
    results::{server_state::ServerState as ServerStateResult, XRPLResponse},
};
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait XRPLAsyncClient: XRPLClient {
    async fn request<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>> {
        self.request_impl(request).await
    }

    async fn get_common_fields(&self) -> Result<CommonFields<'_>> {
        let server_state = self.request(ServerState::new(None).into()).await?;
        let state = server_state
            .try_into_result::<ServerStateResult<'_>>()?
            .state;
        let common_fields = CommonFields {
            network_id: state.network_id,
            build_version: Some(state.build_version),
        };

        Ok(common_fields)
    }
}

impl<T: XRPLClient> XRPLAsyncClient for T {}
