use crate::models::{requests::Request, results::XRPLResponse};
#[cfg(feature = "std")]
use crate::utils::get_random_id;
use alloc::borrow::Cow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(async_fn_in_trait)]
pub trait Client {
    async fn request_impl<
        'a: 'b,
        'b,
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &self,
        request: Req,
    ) -> Result<XRPLResponse<'b, Res, Req>>;
    fn set_request_id<
        'a: 'b,
        'b,
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + Request<'a>,
    >(
        &self,
        request: &mut Req,
    ) -> Cow<'b, str> {
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
