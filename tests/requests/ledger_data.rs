// Scenarios:
//   - base: send a ledger_data request with binary=true and limit=5, verify the response

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::{ledger_data::LedgerData as LedgerDataRequest, LedgerIndex},
    results::ledger_data::LedgerData as LedgerDataResult,
};

#[tokio::test]
async fn test_ledger_data_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let request = LedgerDataRequest::new(
            None,                                       // id
            Some(true),                                 // binary
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(5),                                    // limit
            None,                                       // marker
        );

        let response = client
            .request(request.into())
            .await
            .expect("ledger_data request failed");

        let result: LedgerDataResult = response
            .try_into()
            .expect("failed to parse ledger_data result");

        // Verify response fields
        assert!(!result.ledger_hash.is_empty());
        assert!(result.ledger_index > 0);
        assert_eq!(result.state.len(), 5);

        // Verify each state object has binary data and an index
        for item in result.state.iter() {
            assert!(item.data.is_some());
            assert!(!item.data.as_ref().unwrap().is_empty());
            assert!(!item.index.is_empty());
        }
    })
    .await;
}
