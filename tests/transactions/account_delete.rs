// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/accountDelete.test.ts
//
// Scenarios:
//   - base: submit AccountDelete and expect tecTOO_SOON
//
// NOTE: AccountDelete requires the account's sequence number to be at least 256 lower than the
// current ledger index. A freshly funded account never satisfies this condition on testnet, so
// this test asserts tecTOO_SOON rather than tesSUCCESS (matching xrpl.js behavior — it only
// verifies the transaction is accepted by the node, not that the deletion succeeds).
//
// On Docker standalone mode, call ledger_accept() 256 times before submitting to satisfy the
// condition and assert tesSUCCESS instead.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::account_delete::AccountDelete;

#[tokio::test]
async fn test_account_delete_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let mut tx = AccountDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            destination.classic_address.clone().into(),
            None,
        );

        // submit_and_wait errors on tec* results, so use expect_err.
        // A freshly funded account cannot be deleted until 256 ledgers have closed since its
        // creation (sequence number must be at least 256 below the current ledger index).
        // On Docker standalone, advance 256 ledgers via ledger_accept() to get tesSUCCESS.
        let err = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect_err("Expected tecTOO_SOON — account is too new to delete");

        assert!(
            err.to_string().contains("tecTOO_SOON"),
            "Expected tecTOO_SOON but got: {}",
            err
        );

        ledger_accept().await;
    })
    .await;
}
