// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/escrowFinish.test.ts
//
// Scenarios:
//   - base: create a time-locked XRP escrow then finish it once FinishAfter has passed
//
// NOTE: After EscrowCreate is submitted the test polls `get_ledger_close_time()` until
// the validated ledger's close_time advances past FinishAfter, then submits EscrowFinish.
// The `offer_sequence` is read from the EscrowCreate tx after autofill mutates it in place.

use crate::common::{
    generate_funded_wallet, get_client, get_ledger_close_time, ledger_accept,
    wait_for_ledger_close_time, with_blockchain_lock,
};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::escrow_create::EscrowCreate;
use xrpl::models::transactions::escrow_finish::EscrowFinish;

#[tokio::test]
async fn test_escrow_finish_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
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

        // Wait for the validated ledger close_time to surpass finish_after.
        // rippled validates a finish using the *previous* ledger's close_time,
        // so we wait for close_time > finish_after (not just equal).
        wait_for_ledger_close_time(finish_after as u64 + 1).await;
        ledger_accept().await; // no-op on testnet; advances ledger on Docker standalone

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

        let result = submit_and_wait(
            &mut finish_tx,
            client,
            Some(&wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit EscrowFinish");

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
