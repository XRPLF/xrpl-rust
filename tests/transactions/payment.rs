// Scenarios:
//   - base: XRP payment to a new (unfunded) address

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::{
    models::{transactions::payment::Payment, Amount, XRPAmount},
    wallet::Wallet,
};

#[tokio::test]
async fn test_payment_base() {
    with_blockchain_lock(|| async {
        let sender = generate_funded_wallet().await;
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
            Amount::XRPAmount(XRPAmount::from("20000000")), // 20 XRP — must cover the 20 XRP base reserve in standalone mode
            recipient.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );

        test_transaction(&mut tx, &sender).await;
    })
    .await;
}
