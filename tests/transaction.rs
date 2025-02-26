#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod common;

#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod tests {
    use crate::common::{get_client, get_wallet, with_blockchain_lock};

    use xrpl::{
        asynch::transaction::{sign_and_submit, submit_and_wait},
        models::{
            transactions::{
                account_set::AccountSet, offer_cancel::OfferCancel, offer_create::OfferCreate,
                payment::Payment, trust_set::TrustSet, Memo, Transaction,
            },
            Amount, IssuedCurrencyAmount, XRPAmount,
        },
        wallet::Wallet,
    };

    #[tokio::test]
    async fn test_account_set_transaction() {
        with_blockchain_lock(|| async {
            // Setup client and wallet
            let client = get_client().await;
            let wallet = get_wallet().await;

            // Create an AccountSet transaction to set the domain
            let mut account_set = AccountSet::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some("6578616d706c652e636f6d".into()), // domain (hex for "example.com")
                None,
                None,
                None,
                None,
                None,
                None,
            );

            // Submit and wait for validation
            let result = submit_and_wait(
                &mut account_set,
                client,
                Some(wallet),
                Some(true), // check_fee
                Some(true), // autofill
            )
            .await
            .expect("Failed to submit AccountSet transaction");

            let metadata = result
                .get_transaction_metadata()
                .expect("Expected metadata");
            let result = &metadata.transaction_result;

            assert_eq!(result, "tesSUCCESS");
        })
        .await;
    }

    #[tokio::test]
    async fn test_offer_create_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // Create an offer to trade XRP for USD
            let mut offer = OfferCreate::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Amount::XRPAmount(XRPAmount::from("100")), // taker_pays (100 XRP)
                Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    "USD".into(),
                    "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(), // Bitstamp's issuing address
                    "10".into(),                                // 10 USD
                )),
                None, // expiration
                None, // offer_sequence
            );

            // Sign and submit the transaction
            let result = sign_and_submit(&mut offer, client, wallet, true, true)
                .await
                .expect("Failed to submit OfferCreate transaction");

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_transaction_with_memo() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // Create an AccountSet transaction with a memo
            let mut account_set = AccountSet::new(
                wallet.classic_address.clone().into(), // account
                None,
                None,
                None,
                None,
                Some(vec![Memo::new(
                    Some(hex::encode("Hello, XRPL!").into()), // MemoData (hex encoded)
                    Some(hex::encode("text/plain").into()),   // MemoType (hex encoded)
                    Some(hex::encode("application/json").into()), // MemoFormat (hex encoded)
                )]),
                None,
                None,
                None,
                None,
                None,
                Some("6578616d706c652e636f6d".into()), // domain
                None,
                None,
                None,
                None,
                None,
                None,
            );

            let result = sign_and_submit(&mut account_set, client, wallet, true, true)
                .await
                .expect("Failed to submit transaction with memo");

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_payment_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let sender_wallet = get_wallet().await;
            let receiver_wallet = Wallet::create(None).expect("Failed to create receiver wallet");

            let mut payment = Payment::new(
                sender_wallet.classic_address.clone().into(), // account
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Amount::XRPAmount(XRPAmount::from("10")), // amount
                receiver_wallet.classic_address.clone().into(), // destination
                None,
                None,
                None,
                None,
                None,
            );

            let result = sign_and_submit(&mut payment, client, sender_wallet, true, true)
                .await
                .expect("Failed to submit Payment transaction");

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_trust_set_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            let mut trust_set = TrustSet::new(
                wallet.classic_address.clone().into(), // account
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                IssuedCurrencyAmount::new(
                    // limit_amount
                    "USD".into(),
                    "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(), // Bitstamp's issuing address
                    "1000".into(),                              // Trust line limit
                ),
                None,
                None,
            );

            let result = sign_and_submit(&mut trust_set, client, wallet, true, true)
                .await
                .expect("Failed to submit TrustSet transaction");

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_offer_cancel_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // First create an offer
            let mut offer = OfferCreate::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Amount::XRPAmount(XRPAmount::from("100")), // taker_pays
                Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    "USD".into(),
                    "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                    "10".into(),
                )),
                None,
                None,
            );

            let _ = sign_and_submit(&mut offer, client, wallet, true, true)
                .await
                .expect("Failed to submit OfferCreate transaction");

            // Now cancel the offer using its sequence number
            let mut cancel = OfferCancel::new(
                wallet.classic_address.clone().into(), // account
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                offer.get_common_fields().sequence.unwrap(), // offer_sequence
            );

            let result = sign_and_submit(&mut cancel, client, wallet, true, true)
                .await
                .expect("Failed to submit OfferCancel transaction");

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }
}
