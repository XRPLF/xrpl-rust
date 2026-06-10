// Scenarios:
//   - base: send an account_lines request for a funded wallet and verify
//     the response returns an empty lines list (no trust lines created)

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_lines::AccountLines;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_lines::AccountLines as AccountLinesResult;

#[tokio::test]
async fn test_account_lines_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountLines::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            None,                                       // limit
            None,                                       // peer
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_lines request failed");

        let result: AccountLinesResult = response
            .try_into()
            .expect("failed to parse account_lines result");

        // Verify account matches
        assert_eq!(result.account.as_ref(), wallet.classic_address.as_str());
        // Verify lines is empty (no trust lines created)
        assert!(result.lines.is_empty());
        // Verify ledger_hash exists
        assert!(result.ledger_hash.is_some());
        // Verify ledger_index is valid
        assert!(result.ledger_index.unwrap() > 0);
    })
    .await;
}
