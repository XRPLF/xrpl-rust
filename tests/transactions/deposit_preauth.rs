// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/depositPreauth.test.ts
//
// Scenarios:
//   - base: authorize a second account to send payments to a deposit-auth-enabled account
//
// NOTE: The AuthorizeCredentials / UnauthorizeCredentials scenarios in xrpl.js require the
// Credentials amendment which is not yet enabled on the public testnet. Those variants are
// deferred until Docker standalone mode.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::deposit_preauth::DepositPreauth;

#[tokio::test]
async fn test_deposit_preauth_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let authorized = generate_funded_wallet().await;

        let mut tx = DepositPreauth::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(authorized.classic_address.clone().into()), // authorize
            None,                                            // unauthorize
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit DepositPreauth");

        assert_eq!(
            result
                .get_transaction_metadata()
                .expect("Expected metadata")
                .transaction_result,
            "tesSUCCESS"
        );

        ledger_accept().await;
    })
    .await;
}
