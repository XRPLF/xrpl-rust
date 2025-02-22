mod common;

use common::constants::XRPL_TEST_NET;

use url::Url;
use xrpl::{
    asynch::{
        clients::AsyncJsonRpcClient,
        transaction::{sign_and_submit, submit_and_wait},
        wallet::generate_faucet_wallet,
    },
    models::{
        results::tx::Tx,
        transactions::{
            account_set::AccountSet, offer_cancel::OfferCancel, offer_create::OfferCreate,
            payment::Payment, trust_set::TrustSet, Memo, Transaction,
        },
        Amount, IssuedCurrencyAmount, XRPAmount,
    },
};

#[tokio::test]
async fn test_account_set_transaction() {
    // Setup client and wallet
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate and fund wallet");

    // Create an AccountSet transaction to set the domain
    let mut account_set = AccountSet::new(
        wallet.classic_address.clone().into(),
        None,                                  // fee
        None,                                  // sequence
        None,                                  // account_txn_id
        None,                                  // flags
        None,                                  // last_ledger_sequence
        None,                                  // memos
        None,                                  // signers
        None,                                  // source_tag
        None,                                  // ticket_sequence
        None,                                  // clear_flag
        Some("6578616d706c652e636f6d".into()), // domain (hex for "example.com")
        None,                                  // email_hash
        None,                                  // message_key
        None,                                  // set_flag
        None,                                  // transfer_rate
        None,                                  // tick_size
        None,                                  // nftoken_minter
    );

    // Submit and wait for validation
    let result: Tx = submit_and_wait(
        &mut account_set,
        &client,
        Some(&wallet),
        Some(true), // check_fee
        Some(true), // autofill
    )
    .await
    .expect("Failed to submit AccountSet transaction");

    assert_eq!(
        result
            .meta
            .get("TransactionResult")
            .and_then(|v| v.as_str())
            .unwrap(),
        "tesSUCCESS"
    );
}

#[tokio::test]
async fn test_offer_create_transaction() {
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate and fund wallet");

    // Create an offer to trade XRP for USD
    let mut offer = OfferCreate::new(
        wallet.classic_address.clone().into(),
        None,                                      // fee
        None,                                      // sequence
        None,                                      // account_txn_id
        None,                                      // flags
        None,                                      // last_ledger_sequence
        None,                                      // memos
        None,                                      // signers
        None,                                      // source_tag
        None,                                      // ticket_sequence
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
    let result = sign_and_submit(&mut offer, &client, &wallet, true, true)
        .await
        .expect("Failed to submit OfferCreate transaction");

    assert!(
        result.engine_result_code >= 0,
        "Transaction submission failed"
    );
}

#[tokio::test]
async fn test_transaction_with_memo() {
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate and fund wallet");

    // Create an AccountSet transaction with a memo
    let mut account_set = AccountSet::new(
        wallet.classic_address.clone().into(), // account
        None,                                  // account_txn_id
        None,                                  // fee
        None,                                  // flags
        None,                                  // last_ledger_sequence
        Some(vec![Memo::new(
            Some(hex::encode("Hello, XRPL!").into()), // MemoData (hex encoded)
            Some(hex::encode("text/plain").into()),   // MemoType (hex encoded)
            Some(hex::encode("application/json").into()), // MemoFormat (hex encoded)
        )]),
        None,                                  // sequence
        None,                                  // signers
        None,                                  // source_tag
        None,                                  // ticket_sequence
        None,                                  // clear_flag
        Some("6578616d706c652e636f6d".into()), // domain
        None,                                  // email_hash
        None,                                  // message_key
        None,                                  // set_flag
        None,                                  // transfer_rate
        None,                                  // tick_size
        None,                                  // nftoken_minter
    );

    let result = sign_and_submit(&mut account_set, &client, &wallet, true, true)
        .await
        .expect("Failed to submit transaction with memo");

    assert!(
        result.engine_result_code >= 0,
        "Transaction submission failed"
    );
}

#[tokio::test]
async fn test_payment_transaction() {
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let sender_wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate sender wallet");
    let receiver_wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate receiver wallet");

    let mut payment = Payment::new(
        sender_wallet.classic_address.clone().into(), // account
        None,                                         // account_txn_id
        None,                                         // fee
        None,                                         // flags
        None,                                         // last_ledger_sequence
        None,                                         // memos
        None,                                         // sequence
        None,                                         // signers
        None,                                         // source_tag
        None,                                         // ticket_sequence
        Amount::XRPAmount(XRPAmount::from("10")),     // amount
        receiver_wallet.classic_address.clone().into(), // destination
        None,                                         // deliver_min
        None,                                         // destination_tag
        None,                                         // invoice_id
        None,                                         // paths
        None,                                         // send_max
    );

    let result = sign_and_submit(&mut payment, &client, &sender_wallet, true, true)
        .await
        .expect("Failed to submit Payment transaction");

    assert!(
        result.engine_result_code >= 0,
        "Transaction submission failed"
    );
}

#[tokio::test]
async fn test_trust_set_transaction() {
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate wallet");

    let mut trust_set = TrustSet::new(
        wallet.classic_address.clone().into(), // account
        None,                                  // account_txn_id
        None,                                  // fee
        None,                                  // flags
        None,                                  // last_ledger_sequence
        None,                                  // memos
        None,                                  // sequence
        None,                                  // signers
        None,                                  // source_tag
        None,                                  // ticket_sequence
        IssuedCurrencyAmount::new(
            // limit_amount
            "USD".into(),
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(), // Bitstamp's issuing address
            "1000".into(),                              // Trust line limit
        ),
        None, // quality_in
        None, // quality_out
    );

    let result = sign_and_submit(&mut trust_set, &client, &wallet, true, true)
        .await
        .expect("Failed to submit TrustSet transaction");

    assert!(
        result.engine_result_code >= 0,
        "Transaction submission failed"
    );
}

#[tokio::test]
async fn test_offer_cancel_transaction() {
    let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .expect("Failed to generate wallet");

    // First create an offer
    let mut offer = OfferCreate::new(
        wallet.classic_address.clone().into(),
        None,                                      // fee
        None,                                      // sequence
        None,                                      // account_txn_id
        None,                                      // flags
        None,                                      // last_ledger_sequence
        None,                                      // memos
        None,                                      // signers
        None,                                      // source_tag
        None,                                      // ticket_sequence
        Amount::XRPAmount(XRPAmount::from("100")), // taker_pays
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
            "10".into(),
        )),
        None, // expiration
        None, // offer_sequence
    );

    let _ = sign_and_submit(&mut offer, &client, &wallet, true, true)
        .await
        .expect("Failed to submit OfferCreate transaction");

    // Now cancel the offer using its sequence number
    let mut cancel = OfferCancel::new(
        wallet.classic_address.clone().into(),       // account
        None,                                        // account_txn_id
        None,                                        // fee
        None,                                        // last_ledger_sequence
        None,                                        // memos
        None,                                        // sequence
        None,                                        // signers
        None,                                        // source_tag
        None,                                        // ticket_sequence
        offer.get_common_fields().sequence.unwrap(), // offer_sequence
    );

    let result = sign_and_submit(&mut cancel, &client, &wallet, true, true)
        .await
        .expect("Failed to submit OfferCancel transaction");

    assert!(
        result.engine_result_code >= 0,
        "Transaction submission failed"
    );
}
