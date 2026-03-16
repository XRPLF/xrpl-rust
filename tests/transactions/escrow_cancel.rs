// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/escrowCancel.test.ts
//
// Scenarios:
//   - base: create a time-locked XRP escrow then cancel it once CancelAfter has passed
//
// NOTE: CancelAfter is set to close_time + 3 and FinishAfter to close_time + 2.
// The test waits until the validated ledger close_time exceeds CancelAfter before
// submitting EscrowCancel. An escrow can only be cancelled after CancelAfter passes.

use crate::common::{
    generate_funded_wallet, get_client, get_ledger_close_time, ledger_accept,
    wait_for_ledger_close_time, with_blockchain_lock,
};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::escrow_cancel::EscrowCancel;
use xrpl::models::transactions::escrow_create::EscrowCreate;

#[tokio::test]
async fn test_escrow_cancel_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let close_time = get_ledger_close_time().await;
        let finish_after = (close_time + 2) as u32;
        let cancel_after = (close_time + 3) as u32;

        let mut create_tx = EscrowCreate::new(
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
            Some(cancel_after), // cancel_after
            None,               // condition
            None,               // destination_tag
            Some(finish_after), // finish_after
        );

        submit_and_wait(
            &mut create_tx,
            client,
            Some(&wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit EscrowCreate");

        // offer_sequence = the sequence autofilled into the EscrowCreate transaction
        let offer_sequence = create_tx
            .common_fields
            .sequence
            .expect("Sequence should be autofilled by submit_and_wait");

        // Wait for the validated ledger close_time to surpass cancel_after.
        // rippled validates a cancel using the *previous* ledger's close_time,
        // so we wait for close_time > cancel_after (not just equal).
        wait_for_ledger_close_time(cancel_after as u64 + 1).await;
        ledger_accept().await; // no-op on testnet; advances ledger on Docker standalone

        let mut cancel_tx = EscrowCancel::new(
            wallet.classic_address.clone().into(),
            None,                                  // account_txn_id
            None,                                  // fee
            None,                                  // last_ledger_sequence
            None,                                  // memos
            None,                                  // sequence
            None,                                  // signers
            None,                                  // source_tag
            None,                                  // ticket_sequence
            wallet.classic_address.clone().into(), // owner (= EscrowCreate account)
            offer_sequence,                        // offer_sequence
        );

        let result = submit_and_wait(
            &mut cancel_tx,
            client,
            Some(&wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit EscrowCancel");

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
