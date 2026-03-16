// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/payment.test.ts
//
// Scenarios:
//   - base: XRP payment to a new (unfunded) address

use crate::common::{get_client, get_wallet, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{transactions::payment::Payment, Amount, XRPAmount},
    wallet::Wallet,
};

#[tokio::test]
async fn test_payment_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let sender = get_wallet().await;
        let recipient = Wallet::create(None).expect("Failed to create recipient wallet");

        let mut tx = Payment::new(
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
            Amount::XRPAmount(XRPAmount::from("10000000")), // 10 XRP
            recipient.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );

        let result = submit_and_wait(&mut tx, client, Some(sender), Some(true), Some(true))
            .await
            .expect("Failed to submit Payment");

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
