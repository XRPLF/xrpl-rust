// Scenarios:
//   - base: send a server_state request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::server_state::ServerState as ServerStateRequest,
    results::server_state::ServerState as ServerStateResult,
};

#[tokio::test]
async fn test_server_state_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(ServerStateRequest::new(None).into())
            .await
            .expect("server_state request failed");

        let result: ServerStateResult = response
            .try_into()
            .expect("failed to parse server_state result");

        // Verify essential server_state fields
        assert!(!result.state.build_version.is_empty());

        // Verify validated_ledger is present (standalone always has one)
        assert!(result.state.validated_ledger.is_some());
        let validated = result.state.validated_ledger.unwrap();
        assert!(validated.seq > 0);
    })
    .await;
}
