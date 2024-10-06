use crate::models::{
    requests::{Request, XRPLRequest},
    results::XRPLResponse,
};
use alloc::borrow::Cow;
use anyhow::Result;
use url::Url;

#[allow(async_fn_in_trait)]
pub trait XRPLClient {
    async fn request_impl<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>>;

    fn get_host(&self) -> Url;

    fn set_request_id(&self, request: &mut XRPLRequest<'_>) {
        let common_fields = request.get_common_fields_mut();
        common_fields.id = match &common_fields.id {
            Some(id) => Some(id.clone()),
            None => {
                #[cfg(feature = "std")]
                {
                    Some(self.get_random_id())
                }
                #[cfg(not(feature = "std"))]
                unimplemented!(
                    "Random ID generation is not supported in no_std. Please provide an ID."
                )
            }
        };
    }

    /// Generate a random id.
    #[cfg(feature = "std")]
    fn get_random_id<'a>(&self) -> Cow<'a, str> {
        use alloc::string::ToString;

        let random_id = rand::random::<u32>().to_string();

        Cow::Owned(random_id)
    }
}
