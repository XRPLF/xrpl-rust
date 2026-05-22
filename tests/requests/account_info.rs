// Scenarios:
//   - base: send an account_info request for a funded wallet and verify the response

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_info::AccountInfo;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_info::AccountInfoVersionMap;

#[tokio::test]
async fn test_account_info_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountInfo::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(true),                                 // strict
            None,                                       // queue
            None,                                       // signer_lists
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_info request failed");

        let result: AccountInfoVersionMap = response
            .try_into()
            .expect("failed to parse account_info result");

        let account_data = result.get_account_root();

        // Verify account matches
        assert_eq!(
            account_data.account.as_ref(),
            wallet.classic_address.as_str()
        );
        // Verify balance is 400 XRP (400000000 drops)
        assert_eq!(
            account_data.balance.as_ref().map(|b| b.0.as_ref()),
            Some("400000000")
        );
        // Verify owner count
        assert_eq!(account_data.owner_count, 0);
        // Verify sequence is a valid number
        assert!(account_data.sequence > 0);
        // Verify PreviousTxnID exists
        assert!(!account_data.previous_txn_id.is_empty());
    })
    .await;
}
