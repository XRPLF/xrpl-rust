// Scenarios:
//   - base: send a random request and verify it returns a 64-character hex string

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::random::Random as RandomRequest, results::random::Random as RandomResult,
};

#[tokio::test]
async fn test_random_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(RandomRequest::new(None).into())
            .await
            .expect("random request failed");

        let result: RandomResult = response
            .try_into()
            .expect("failed to parse random result");

        // Verify the random string is a 64-character hex value
        assert_eq!(result.random.len(), 64);
        assert!(result.random.chars().all(|c| c.is_ascii_hexdigit()));
    })
    .await;
}

