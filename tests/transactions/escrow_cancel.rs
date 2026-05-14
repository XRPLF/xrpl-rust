// Scenarios:
//   - base: create a time-locked XRP escrow then cancel it once CancelAfter has passed
//
// NOTE: CancelAfter is set to close_time + 3 and FinishAfter to close_time + 2.
// After EscrowCreate the test:
//   1. Queries account_objects to confirm the escrow exists on-chain
//   2. Looks up the creating tx to get the validated Sequence (OfferSequence)
//   3. Waits for close_time >= CancelAfter, then one more ledger_accept
// An escrow can only be cancelled after CancelAfter passes.

use crate::common::{
    generate_funded_wallet, get_escrow_offer_sequence, get_ledger_close_time, ledger_accept,
    test_transaction, wait_for_ledger_close_time, with_blockchain_lock,
};
use xrpl::models::transactions::escrow_cancel::EscrowCancel;
use xrpl::models::transactions::escrow_create::EscrowCreate;

#[tokio::test]
async fn test_escrow_cancel_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let close_time = get_ledger_close_time().await;
        let finish_after = (close_time + 2) as u32;
        let cancel_after = (close_time + 3) as u32;

        let mut create_tx = EscrowCreate::new(
            wallet.classic_address.clone().into(),
            None,                                       // account_txn_id
            None,                                       // fee
            None,                                       // last_ledger_sequence
            None,                                       // memos
            None,                                       // sequence
            None,                                       // signers
            None,                                       // source_tag
            None,                                       // ticket_sequence
            "10000".into(),                             // amount: 10 000 drops
            destination.classic_address.clone().into(), // destination
            Some(cancel_after),                         // cancel_after
            None,                                       // condition
            None,                                       // destination_tag
            Some(finish_after),                         // finish_after
        );

        // test_transaction signs, submits, asserts tesSUCCESS, and calls ledger_accept.
        test_transaction(&mut create_tx, &wallet).await;

        // Look up the validated Sequence via account_objects → tx query
        // instead of reading the autofilled value from the tx struct.  This confirms the
        // escrow actually exists on-chain before we try to cancel it.
        let offer_sequence = get_escrow_offer_sequence(&wallet.classic_address).await;

        // Wait for the validated ledger close_time to reach CancelAfter.
        wait_for_ledger_close_time(cancel_after as u64).await;
        // rippled validates a cancel using the *previous* ledger's close_time,
        // so one more ledger_accept ensures that previous close_time > CancelAfter.
        ledger_accept().await;

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

        test_transaction(&mut cancel_tx, &wallet).await;
    })
    .await;
}
