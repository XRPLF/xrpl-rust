use crate::models::{
    requests::{Request, XRPLRequest},
    results::XRPLResponse,
};
#[cfg(feature = "std")]
use crate::utils::get_random_id;
use alloc::borrow::{Cow, ToOwned};
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait Client {
    async fn request_impl<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>>;

    fn set_request_id(&self, request: &mut XRPLRequest<'_>) -> () {
        let common_fields = request.get_common_fields_mut();
        common_fields.id = match &common_fields.id {
            Some(id) => Some(id.to_owned()),
            None => {
                #[cfg(feature = "std")]
                {
                    let mut rng = rand::thread_rng();
                    Some(Cow::Owned(get_random_id(&mut rng)))
                }
                #[cfg(not(feature = "std"))]
                unimplemented!("get_random_id is not yet implemented for no_std. Please provide an `id` in the request.");
            }
        };
    }
}
