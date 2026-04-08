// Scenarios:
//   - base: send a ping request and verify the response is successful

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::ping::Ping as PingRequest;

#[tokio::test]
async fn test_ping_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(PingRequest::new(None).into())
            .await
            .expect("ping request failed");

        // Ping response is minimal — verify the response is successful
        assert!(response.is_success());
        assert!(response.error.is_none());
    })
    .await;
}
