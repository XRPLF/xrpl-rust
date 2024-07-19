use crate::models::{
    requests::{Request, XRPLRequest},
    results::XRPLResponse,
};
#[cfg(feature = "std")]
use crate::utils::get_random_id;
use alloc::borrow::Cow;
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait Client {
    async fn request_impl<'a: 'b, 'b>(&self, request: XRPLRequest<'a>) -> Result<XRPLResponse<'b>>;
    fn set_request_id<'a: 'b, 'b>(&self, request: &mut XRPLRequest<'a>) -> Cow<'b, str> {
        let common_fields = request.get_common_fields();
        let request_id: Cow<'_, str> = match common_fields.id.clone() {
            Some(id) => id,
            None => {
                #[cfg(feature = "std")]
                {
                    let mut rng = rand::thread_rng();
                    Cow::Owned(get_random_id(&mut rng))
                }
                #[cfg(not(feature = "std"))]
                unimplemented!("get_random_id is not yet implemented for no_std. Please provide an `id` in the request.");
            }
        };
        request.get_common_fields_mut().id = Some(request_id.clone());
        request_id
    }
}
