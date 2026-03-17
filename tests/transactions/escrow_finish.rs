// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/escrowFinish.test.ts
//
// Scenarios:
//   - base: create a time-locked XRP escrow then finish it once FinishAfter has passed
//
// NOTE: After EscrowCreate is submitted the test:
//   1. Queries account_objects to confirm the escrow exists on-chain
//   2. Looks up the creating tx to get the validated Sequence (OfferSequence)
//   3. Waits for close_time >= FinishAfter, then one more ledger_accept
// This mirrors the xrpl.js pattern exactly (account_objects → tx lookup).

use crate::common::{
    generate_funded_wallet, get_escrow_offer_sequence, get_ledger_close_time, ledger_accept,
    test_transaction, wait_for_ledger_close_time, with_blockchain_lock,
};
use xrpl::models::transactions::escrow_create::EscrowCreate;
use xrpl::models::transactions::escrow_finish::EscrowFinish;

#[tokio::test]
async fn test_escrow_finish_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let close_time = get_ledger_close_time().await;
        let finish_after = (close_time + 2) as u32;

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
            None,               // cancel_after
            None,               // condition
            None,               // destination_tag
            Some(finish_after), // finish_after
        );

        // test_transaction signs, submits, asserts tesSUCCESS, and calls ledger_accept.
        test_transaction(&mut create_tx, &wallet).await;

        // Mirroring xrpl.js: look up the validated Sequence via account_objects → tx query
        // instead of reading the autofilled value from the tx struct.  This confirms the
        // escrow actually exists on-chain before we try to finish it.
        let offer_sequence =
            get_escrow_offer_sequence(&wallet.classic_address).await;

        // Wait for the validated ledger close_time to reach FinishAfter (mirrors
        // xrpl.js waitForAndForceProgressLedgerTime(CLOSE_TIME + 2)).
        wait_for_ledger_close_time(finish_after as u64).await;
        // rippled validates a finish using the *previous* ledger's close_time,
        // so one more ledger_accept ensures that previous close_time > FinishAfter.
        ledger_accept().await;

        let mut finish_tx = EscrowFinish::new(
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
            None,                                  // condition
            None,                                  // fulfillment
        );

        test_transaction(&mut finish_tx, &wallet).await;
    })
    .await;
}
