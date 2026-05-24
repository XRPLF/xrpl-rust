use crate::models::{
    requests::{Request, XRPLRequest},
    results::XRPLResponse,
};
use url::Url;

use super::exceptions::XRPLClientResult;

#[allow(async_fn_in_trait)]
pub trait XRPLClient {
    async fn request_impl<'a: 'b, 'b>(
        &self,
        request: XRPLRequest<'a>,
    ) -> XRPLClientResult<XRPLResponse<'b>>;

    fn get_host(&self) -> Url;

    fn set_request_id(&self, request: &mut XRPLRequest<'_>) {
        let common_fields = request.get_common_fields_mut();
        if common_fields.id.is_none() {
            #[cfg(feature = "std")]
            {
                common_fields.id = Some(self.get_random_id());
            }
        }
    }

    /// Generate a random id.
    #[cfg(feature = "std")]
    fn get_random_id<'a>(&self) -> alloc::borrow::Cow<'a, str> {
        use alloc::string::ToString;

        let random_id = rand::random::<u32>().to_string();

        alloc::borrow::Cow::Owned(random_id)
    }
}

#[cfg(all(test, not(feature = "std")))]
mod tests {
    use super::XRPLClient;
    use crate::{
        asynch::clients::exceptions::XRPLClientResult,
        models::{
            requests::{random::Random, Request, XRPLRequest},
            results::XRPLResponse,
        },
    };
    use url::Url;

    struct TestClient;

    impl XRPLClient for TestClient {
        async fn request_impl<'a: 'b, 'b>(
            &self,
            _request: XRPLRequest<'a>,
        ) -> XRPLClientResult<XRPLResponse<'b>> {
            unreachable!("not used in this test")
        }

        fn get_host(&self) -> Url {
            Url::parse("ws://localhost:6006").expect("valid URL")
        }
    }

    #[test]
    fn set_request_id_no_std_does_not_panic_or_generate_id() {
        let client = TestClient;
        let mut request: XRPLRequest<'_> = Random::new(None).into();

        client.set_request_id(&mut request);

        assert!(request.get_common_fields().id.is_none());
    }
}
