// Scenarios:
//   - base: send a server_info request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::server_info::ServerInfo as ServerInfoRequest,
    results::server_info::ServerInfo as ServerInfoResult,
};

#[tokio::test]
async fn test_server_info_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(ServerInfoRequest::new(None).into())
            .await
            .expect("server_info request failed");

        let result: ServerInfoResult = response
            .try_into()
            .expect("failed to parse server_info result");

        // Verify essential server_info fields
        assert!(!result.info.build_version.is_empty());
        assert!(!result.info.complete_ledgers.is_empty());
        assert!(result.info.load_factor > 0);

        // Verify validated_ledger is present (standalone always has one)
        assert!(result.info.validated_ledger.is_some());
        let validated = result.info.validated_ledger.unwrap();
        assert!(validated.seq > 0);
    })
    .await;
}
