// Scenarios:
//   - base: send an account_currencies request for a funded wallet and verify
//     the response returns empty currency lists (no trust lines created)

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_currencies::AccountCurrencies;
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_currencies::AccountCurrencies as AccountCurrenciesResult;

#[tokio::test]
async fn test_account_currencies_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet = crate::common::generate_funded_wallet().await;

        let request = AccountCurrencies::new(
            None,                                       // id
            wallet.classic_address.clone().into(),      // account
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(true),                                 // strict
        );

        let response = client
            .request(request.into())
            .await
            .expect("account_currencies request failed");

        let result: AccountCurrenciesResult = response
            .try_into()
            .expect("failed to parse account_currencies result");

        // Verify currencies are empty (no trust lines)
        assert!(result.receive_currencies.is_empty());
        assert!(result.send_currencies.is_empty());
        // Verify validated
        assert!(result.validated);
        // Verify ledger_hash exists
        assert!(result.ledger_hash.is_some());
        // Verify ledger_index is valid
        assert!(result.ledger_index > 0);
    })
    .await;
}
