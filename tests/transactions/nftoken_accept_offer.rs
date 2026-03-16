// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//   (xrpl.js covers NFTokenAcceptOffer indirectly via the "test with Amount" scenario in nftokenMint)
//
// Scenarios:
//   - sell_offer: seller mints a transferable NFT, creates a sell offer, buyer accepts it

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{
        results::nftoken::{NFTokenCreateOfferResult, NFTokenMintResult},
        transactions::{
            nftoken_accept_offer::NFTokenAcceptOffer,
            nftoken_create_offer::{NFTokenCreateOffer, NFTokenCreateOfferFlag},
            nftoken_mint::{NFTokenMint, NFTokenMintFlag},
        },
        Amount, XRPAmount,
    },
};

const TEST_NFT_URL: &str = "https://example.com/nft.json";

#[tokio::test]
async fn test_nftoken_accept_offer_sell() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let seller = generate_funded_wallet().await;
        let buyer = generate_funded_wallet().await;

        // Step 1: seller mints an NFT with TfTransferable so it can change hands.
        // NOTE: NFTokenMint places `flags` at position 4 (after fee, before last_ledger_sequence),
        // which differs from most other transactions where flags follow the common fields.
        let mut mint = NFTokenMint::new(
            seller.classic_address.clone().into(),
            None,                                                             // account_txn_id
            None,                                                             // fee
            Some(vec![NFTokenMintFlag::TfTransferable].into()),               // flags (position 4!)
            None,                                                             // last_ledger_sequence
            None,                                                             // memos
            None,                                                             // sequence
            None,                                                             // signers
            None,                                                             // source_tag
            None,                                                             // ticket_sequence
            0,                                                                // nftoken_taxon
            None,                                                             // issuer
            None,                                                             // transfer_fee
            Some(hex::encode(TEST_NFT_URL).into()),                           // uri
        );

        let mint_result =
            submit_and_wait(&mut mint, client, Some(&seller), Some(true), Some(true))
                .await
                .expect("Failed to mint NFT");

        let nftoken_id = NFTokenMintResult::try_from(mint_result)
            .expect("Failed to extract NFTokenID")
            .nftoken_id
            .to_string();

        ledger_accept().await;

        // Step 2: seller creates a sell offer (destination = buyer).
        let mut create_offer = NFTokenCreateOffer::new(
            seller.classic_address.clone().into(),
            None,
            None,
            Some(vec![NFTokenCreateOfferFlag::TfSellOffer].into()),
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")), // 1 XRP
            nftoken_id.into(),
            Some(buyer.classic_address.clone().into()), // destination
            None,
            None,
        );

        let offer_result =
            submit_and_wait(&mut create_offer, client, Some(&seller), Some(true), Some(true))
                .await
                .expect("Failed to create NFT sell offer");

        let offer_id = NFTokenCreateOfferResult::try_from(offer_result)
            .expect("Failed to extract OfferID")
            .offer_id
            .to_string();

        ledger_accept().await;

        // Step 3: buyer accepts the sell offer.
        let mut accept = NFTokenAcceptOffer::new(
            buyer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(offer_id.into()), // nftoken_sell_offer
            None,                  // nftoken_buy_offer
            None,                  // nftoken_broker_fee
        );

        let result = submit_and_wait(&mut accept, client, Some(&buyer), Some(true), Some(true))
            .await
            .expect("Failed to submit NFTokenAcceptOffer");

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
