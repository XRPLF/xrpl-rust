use crate::{
    models::{requests::Request, results::XRPLResponse},
    utils::get_random_id,
};
use alloc::borrow::Cow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub(crate) trait Client {
    async fn request_impl<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + for<'a> Request<'a>,
    >(
        &self,
        request: Req,
    ) -> Result<XRPLResponse<'_, Res, Req>>;
    fn set_request_id<
        Res: Serialize + for<'de> Deserialize<'de>,
        Req: Serialize + for<'de> Deserialize<'de> + for<'a> Request<'a>,
    >(
        &self,
        request: &mut Req,
    ) -> Cow<'_, str> {
        let common_fields = request.get_common_fields();
        let request_id: Cow<'_, str> = match common_fields.id.clone() {
            Some(id) => id,
            None => {
                #[cfg(feature = "std")]
                let mut rng = rand::thread_rng();
                Cow::Owned(get_random_id(&mut rng))
            }
        };
        request.get_common_fields_mut().id = Some(request_id.clone());
        request_id
    }
}
