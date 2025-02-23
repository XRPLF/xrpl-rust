use super::{client::XRPLClient, exceptions::XRPLClientResult, CommonFields};
use crate::models::{
    requests::{server_state::ServerState, XRPLRequest},
    results::{server_state::ServerState as ServerStateResult, XRPLResponse},
};

#[allow(async_fn_in_trait)]
pub trait XRPLAsyncClient: XRPLClient {
    async fn request<'a: 'b, 'b>(
        &self,
        request: XRPLRequest<'a>,
    ) -> XRPLClientResult<XRPLResponse<'b>> {
        self.request_impl(request).await
    }

    async fn get_common_fields(&self) -> XRPLClientResult<CommonFields<'_>> {
        let server_state = self.request(ServerState::new(None).into()).await?;
        let server_state: ServerStateResult = server_state.try_into()?;
        let common_fields = CommonFields {
            network_id: None, // TODO Server state has no network ID.
            build_version: Some(state.build_version),
        };

        Ok(common_fields)
    }
}

impl<T: XRPLClient> XRPLAsyncClient for T {}
