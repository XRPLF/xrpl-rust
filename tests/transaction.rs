#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod common;

#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod tests {
    use crate::common::{get_client, get_wallet, with_blockchain_lock};

    use xrpl::{
        asynch::{
            clients::XRPLAsyncClient,
            transaction::{sign_and_submit, submit_and_wait},
        },
        models::{
            requests::ledger::Ledger as LedgerRequest,
            results::ledger::Ledger as LedgerResult,
            transactions::{
                account_set::AccountSet, check_create::CheckCreate,
                deposit_preauth::DepositPreauth, escrow_create::EscrowCreate,
                nftoken_mint::NFTokenMint, offer_cancel::OfferCancel, offer_create::OfferCreate,
                payment::Payment, set_regular_key::SetRegularKey, ticket_create::TicketCreate,
                trust_set::TrustSet, Memo, Transaction,
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

            // Get hash from TxVersionMap
            let tx_hash = match &result {
                xrpl::models::results::tx::TxVersionMap::Default(tx) => tx.base.hash.clone(),
                xrpl::models::results::tx::TxVersionMap::V1(tx) => tx.base.hash.clone(),
            };
            println!("✅ AccountSet transaction succeeded - hash: {}", tx_hash);

            let metadata = result
                .get_transaction_metadata()
                .expect("Expected metadata");
            let tx_result = &metadata.transaction_result;

            assert_eq!(tx_result, "tesSUCCESS");
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

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ OfferCreate transaction succeeded - hash: {}", tx_hash);

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

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!(
                "✅ AccountSet with Memo transaction succeeded - hash: {}",
                tx_hash
            );

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

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ Payment transaction succeeded - hash: {}", tx_hash);

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

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ TrustSet transaction succeeded - hash: {}", tx_hash);

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

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ OfferCancel transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "Transaction submission failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_escrow_create_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;
            let destination_wallet =
                Wallet::create(None).expect("Failed to create destination wallet");

            // Get ledger close_time for FinishAfter
            let ledger_request = LedgerRequest::new(
                None,
                None,
                None,
                None,
                None,
                None,
                Some("validated".into()),
                None,
                None,
                None,
            );
            let ledger_response = client
                .request(ledger_request.into())
                .await
                .expect("Failed to get ledger");
            let ledger_result: LedgerResult = ledger_response
                .try_into()
                .expect("Failed to parse ledger result");
            let close_time = ledger_result.ledger.close_time;

            // Create escrow with FinishAfter in the future
            let mut escrow = EscrowCreate::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                XRPAmount::from("10000"), // amount in drops
                destination_wallet.classic_address.clone().into(),
                None,                        // cancel_after
                None,                        // condition
                None,                        // destination_tag
                Some(close_time as u32 + 5), // finish_after
            );

            let result = sign_and_submit(&mut escrow, client, wallet, true, true)
                .await
                .expect("Failed to submit EscrowCreate transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ EscrowCreate transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "EscrowCreate transaction failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_check_create_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;
            let destination_wallet =
                Wallet::create(None).expect("Failed to create destination wallet");

            // Create a check for XRP
            let mut check = CheckCreate::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                destination_wallet.classic_address.clone().into(), // destination
                Amount::XRPAmount(XRPAmount::from("10000000")),    // send_max: 10 XRP
                None,                                              // destination_tag
                None,                                              // expiration
                None,                                              // invoice_id
            );

            let result = sign_and_submit(&mut check, client, wallet, true, true)
                .await
                .expect("Failed to submit CheckCreate transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ CheckCreate transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "CheckCreate transaction failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_nftoken_mint_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // Mint an NFT with URI (hex encoded)
            let uri = hex::encode("https://example.com/nft/1");
            let mut mint = NFTokenMint::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None, // flags
                None,
                None,
                None,
                None,
                None,
                None,
                0,                // nftoken_taxon
                None,             // issuer
                None,             // transfer_fee
                Some(uri.into()), // uri
            );

            let result = sign_and_submit(&mut mint, client, wallet, true, true)
                .await
                .expect("Failed to submit NFTokenMint transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ NFTokenMint transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "NFTokenMint transaction failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_deposit_preauth_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;
            let authorize_wallet =
                Wallet::create(None).expect("Failed to create wallet to authorize");

            // Preauthorize another account for deposits
            let mut preauth = DepositPreauth::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(authorize_wallet.classic_address.clone().into()), // authorize
                None,                                                  // unauthorize
            );

            let result = sign_and_submit(&mut preauth, client, wallet, true, true)
                .await
                .expect("Failed to submit DepositPreauth transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!(
                "✅ DepositPreauth transaction succeeded - hash: {}",
                tx_hash
            );

            assert!(
                result.engine_result_code >= 0,
                "DepositPreauth transaction failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_set_regular_key_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;
            let regular_key_wallet =
                Wallet::create(None).expect("Failed to create regular key wallet");

            // Set a regular key for the account
            let mut set_key = SetRegularKey::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(regular_key_wallet.classic_address.clone().into()), // regular_key
            );

            let result = sign_and_submit(&mut set_key, client, wallet, true, true)
                .await
                .expect("Failed to submit SetRegularKey transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ SetRegularKey transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "SetRegularKey transaction failed"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_ticket_create_transaction() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // Create 5 tickets for future use
            let mut ticket = TicketCreate::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                5, // ticket_count
            );

            let result = sign_and_submit(&mut ticket, client, wallet, true, true)
                .await
                .expect("Failed to submit TicketCreate transaction");

            let tx_hash = result
                .tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown");
            println!("✅ TicketCreate transaction succeeded - hash: {}", tx_hash);

            assert!(
                result.engine_result_code >= 0,
                "TicketCreate transaction failed"
            );
        })
        .await;
    }
}
