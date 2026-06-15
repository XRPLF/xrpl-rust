// Scenarios:
//   - base: send a ledger request and verify the response fields

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::ledger::Ledger as LedgerRequest, results::ledger::Ledger as LedgerResult,
};

#[tokio::test]
async fn test_ledger_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let response = client
            .request(
                LedgerRequest::new(
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("validated".into()),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("ledger request failed");

        let result: LedgerResult = response.try_into().expect("failed to parse ledger result");

        // Verify the response contains valid ledger data
        assert!(!result.ledger_hash.is_empty());
        assert!(result.ledger_index > 0);
        assert!(result.validated == Some(true));

        // Verify ledger inner fields
        assert!(!result.ledger.account_hash.is_empty());
        assert!(!result.ledger.ledger_hash.is_empty());
        assert!(result.ledger.close_time > 0);
        assert!(result.ledger.closed);
    })
    .await;
}
