// Scenarios:
//   - base: send an account_objects request for a funded wallet and verify
//     the response returns an empty account_objects list

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::AccountObjects;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_objects::AccountObjects as AccountObjectsResult;

#[tokio::test]
async fn test_account_objects_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountObjects::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            None,                                       // type
            None,                                       // deletion_blockers_only
            None,                                       // limit
            None,                                       // marker
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_objects request failed");

        let result: AccountObjectsResult = response
            .try_into()
            .expect("failed to parse account_objects result");

        // Verify account matches
        assert_eq!(result.account.as_ref(), wallet.classic_address.as_str());
        // Verify account_objects is empty
        assert!(result.account_objects.is_empty());
        // Verify ledger_hash exists
        assert!(result.ledger_hash.is_some());
        // Verify ledger_index is valid
        assert!(result.ledger_index.unwrap() > 0);
    })
    .await;
}
