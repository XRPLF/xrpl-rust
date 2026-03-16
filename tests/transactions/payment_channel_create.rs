// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/paymentChannelCreate.test.ts
//
// Scenarios:
//   - base: create a payment channel from sender to destination with 100 drops and 86400s settle delay

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::payment_channel_create::PaymentChannelCreate;
use xrpl::models::XRPAmount;

#[tokio::test]
async fn test_payment_channel_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let mut tx = PaymentChannelCreate::new(
            wallet.classic_address.clone().into(),
            None,                                               // account_txn_id
            None,                                               // fee
            None,                                               // last_ledger_sequence
            None,                                               // memos
            None,                                               // sequence
            None,                                               // signers
            None,                                               // source_tag
            None,                                               // ticket_sequence
            XRPAmount::from("100"),                             // amount: 100 drops
            destination.classic_address.clone().into(),         // destination
            wallet.public_key.clone().into(),                   // public_key (hex)
            86400,                                              // settle_delay: 1 day
            None,                                               // cancel_after
            None,                                               // destination_tag
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit PaymentChannelCreate");

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
