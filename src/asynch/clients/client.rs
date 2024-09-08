use crate::models::{
    requests::{Request, XRPLRequest},
    results::XRPLResponse,
};
use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
};
use anyhow::Result;
use rand::Rng;
use url::Url;

/// Generate a random id.
pub fn get_random_id<T: rand::RngCore>(rng: &mut T) -> String {
    let id: u32 = rng.gen();
    id.to_string()
}

#[allow(async_fn_in_trait)]
pub trait Client {
    async fn request_impl<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>>;

    fn get_host(&self) -> Url;

    fn set_request_id(&self, request: &mut XRPLRequest<'_>) {
        let common_fields = request.get_common_fields_mut();
        common_fields.id = match &common_fields.id {
            Some(id) => Some(id.to_owned()),
            None => {
                #[cfg(feature = "std")]
                {
                    use alloc::borrow::Cow;
                    let mut rng = rand::thread_rng();
                    Some(Cow::Owned(get_random_id(&mut rng)))
                }
                #[cfg(not(feature = "std"))]
                unimplemented!("get_random_id is not yet implemented for no_std. Please provide an `id` in the request.");
            }
        };
    }
}
