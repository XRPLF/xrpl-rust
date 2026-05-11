// Direct end-to-end test of asynch::transaction::submit_and_wait.
//
// Other transaction tests use the shared `test_transaction` helper, which calls
// `sign_and_submit` and then `ledger_accept`. submit_and_wait does its own polling
// for ledger validation, so it needs a separate test. Standalone rippled does not
// auto-close ledgers, so a background task drives `ledger_accept` while the poll
// loop runs.

use core::time::Duration;

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{transactions::payment::Payment, Amount, XRPAmount},
    wallet::Wallet,
};

#[tokio::test]
async fn test_submit_and_wait_payment() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let sender = generate_funded_wallet().await;
        let recipient = Wallet::create(None).expect("recipient wallet");

        let mut payment = Payment::new(
            sender.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("20000000")), // 20 XRP — covers the standalone base reserve
            recipient.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );

        // Drive ledger closes while submit_and_wait polls for validation.
        let ledger_driver = tokio::spawn(async {
            loop {
                ledger_accept().await;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        let validated_tx =
            submit_and_wait(&mut payment, client, Some(&sender), Some(true), Some(true))
                .await
                .expect("submit_and_wait should return a validated transaction");

        ledger_driver.abort();
        let _ = ledger_driver.await;

        let metadata = validated_tx
            .get_transaction_metadata()
            .expect("validated transaction should have metadata");

        assert_eq!(metadata.transaction_result, "tesSUCCESS");
    })
    .await;
}
