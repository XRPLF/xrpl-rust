// Scenarios:
//   - base: send a ledger_current request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::ledger_current::LedgerCurrent as LedgerCurrentRequest,
    results::ledger_current::LedgerCurrent as LedgerCurrentResult,
};

#[tokio::test]
async fn test_ledger_current_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(LedgerCurrentRequest::new(None).into())
            .await
            .expect("ledger_current request failed");

        let result: LedgerCurrentResult = response
            .try_into()
            .expect("failed to parse ledger_current result");

        // Verify the response contains a valid ledger current index
        assert!(result.ledger_current_index > 0);
    })
    .await;
}
