// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/paymentChannelCreate.test.ts
//
// Scenarios:
//   - base: create a payment channel from sender to destination with 100 drops and 86400s settle delay

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::payment_channel_create::PaymentChannelCreate;
use xrpl::models::XRPAmount;

#[tokio::test]
async fn test_payment_channel_create_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let mut tx = PaymentChannelCreate::new(
            wallet.classic_address.clone().into(),
            None,                                       // account_txn_id
            None,                                       // fee
            None,                                       // last_ledger_sequence
            None,                                       // memos
            None,                                       // sequence
            None,                                       // signers
            None,                                       // source_tag
            None,                                       // ticket_sequence
            XRPAmount::from("100"),                     // amount: 100 drops
            destination.classic_address.clone().into(), // destination
            wallet.public_key.clone().into(),           // public_key (hex)
            86400,                                      // settle_delay: 1 day
            None,                                       // cancel_after
            None,                                       // destination_tag
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}
