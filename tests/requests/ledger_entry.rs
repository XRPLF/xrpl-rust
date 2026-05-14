// Scenarios:
//   - base: get an entry index from ledger_data, then query ledger_entry with that index

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::{ledger_data::LedgerData as LedgerDataRequest, LedgerIndex},
    results::ledger_data::LedgerData as LedgerDataResult,
};

#[tokio::test]
async fn test_ledger_entry_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        // First, get a valid entry index from ledger_data
        let data_request = LedgerDataRequest::new(
            None,                                       // id
            None,                                       // binary
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(1),                                    // limit
            None,                                       // marker
        );

        let data_response = client
            .request(data_request.into())
            .await
            .expect("ledger_data request failed");

        let data_result: LedgerDataResult = data_response
            .try_into()
            .expect("failed to parse ledger_data result");

        let entry_index = data_result.state[0].index.clone();

        // Now query ledger_entry with that index
        let entry_request = xrpl::models::requests::ledger_entry::LedgerEntry::new(
            None,                      // id
            None,                      // account_root
            None,                      // binary
            None,                      // check
            None,                      // deposit_preauth
            None,                      // directory
            None,                      // escrow
            Some(entry_index.clone()), // index
            None,                      // ledger_hash
            None,                      // ledger_index
            None,                      // offer
            None,                      // payment_channel
            None,                      // ripple_state
            None,                      // ticket
        );

        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("failed ledger_entry request");

        let entry_result: xrpl::models::results::ledger_entry::LedgerEntry = entry_response
            .try_into()
            .expect("failed to parse ledger_entry result");

        // Verify the returned index matches what we requested
        assert_eq!(entry_result.index.as_ref(), entry_index.as_ref());
        // Verify the node is present (non-binary mode)
        assert!(entry_result.node.is_some());
    })
    .await;
}
