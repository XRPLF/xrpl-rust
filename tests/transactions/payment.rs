// Scenarios:
//   - base: XRP payment to a new (unfunded) address
//   - with_credential_ids: Payment to DepositAuth-gated destination via credential authorization

use crate::common::{
    generate_funded_wallet, provision_credential_for_destination, submit_tx, test_transaction,
    with_blockchain_lock, SubmitOptions, CREDENTIAL_TYPE_KYC,
};
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

// ── with_credential_ids: Payment to DepositAuth-gated destination ─────────────

#[tokio::test]
async fn test_payment_with_credential_ids() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let credential_hash = provision_credential_for_destination(
            &issuer,
            &subject,
            &destination,
            CREDENTIAL_TYPE_KYC,
        )
        .await;

        // Step 1: payment WITHOUT credentials — must be rejected by DepositAuth gate.
        let mut neg_tx = Payment::new(
            subject.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")),
            destination.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );
        let neg_result = submit_tx(
            &mut neg_tx,
            SubmitOptions {
                wallet: &subject,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            neg_result, "tecNO_PERMISSION",
            "payment without credential_ids should be rejected when destination has DepositAuth"
        );

        // Step 2: payment WITH credential_ids — must succeed.
        let mut tx = Payment::new(
            subject.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")),
            destination.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );
        tx.credential_ids = Some(vec![credential_hash.into()]);

        test_transaction(&mut tx, &subject).await;
    })
    .await;
}
