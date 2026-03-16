// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/escrowCreate.test.ts
//
// Scenarios:
//   - base: create a time-locked XRP escrow (FinishAfter = close_time + 2) and verify tesSUCCESS
//
// NOTE: FinishAfter is set slightly ahead of the current ledger close_time.
// On testnet ledgers close automatically every ~3-4 s, so EscrowFinish and EscrowCancel
// scenarios (which require waiting for time to advance) live in their own test files.

use crate::common::{
    generate_funded_wallet, get_client, get_ledger_close_time, ledger_accept, with_blockchain_lock,
};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::escrow_create::EscrowCreate;

#[tokio::test]
async fn test_escrow_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let close_time = get_ledger_close_time().await;
        let finish_after = (close_time + 2) as u32;

        let mut tx = EscrowCreate::new(
            wallet.classic_address.clone().into(),
            None,           // account_txn_id
            None,           // fee
            None,           // last_ledger_sequence
            None,           // memos
            None,           // sequence
            None,           // signers
            None,           // source_tag
            None,           // ticket_sequence
            "10000".into(), // amount: 10 000 drops
            destination.classic_address.clone().into(), // destination
            None,               // cancel_after
            None,               // condition
            None,               // destination_tag
            Some(finish_after), // finish_after
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit EscrowCreate");

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
