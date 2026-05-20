// Scenarios:
//   - base: send a ledger_closed request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::ledger_closed::LedgerClosed as LedgerClosedRequest,
    results::ledger_closed::LedgerClosed as LedgerClosedResult,
};

#[tokio::test]
async fn test_ledger_closed_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(LedgerClosedRequest::new(None).into())
            .await
            .expect("ledger_closed request failed");

        let result: LedgerClosedResult = response
            .try_into()
            .expect("failed to parse ledger_closed result");

        // Verify the response contains a valid ledger hash and index
        assert!(!result.ledger_hash.is_empty());
        assert!(result.ledger_index > 0);
    })
    .await;
}
