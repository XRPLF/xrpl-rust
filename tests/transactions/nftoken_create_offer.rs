// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//
// Scenarios:
//   - sell_offer: mint an NFT then create a sell offer for it

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{
        results::nftoken::NFTokenMintResult,
        transactions::{
            nftoken_create_offer::{NFTokenCreateOffer, NFTokenCreateOfferFlag},
            nftoken_mint::NFTokenMint,
        },
        Amount, XRPAmount,
    },
};

const TEST_NFT_URL: &str = "https://example.com/nft.json";

#[tokio::test]
async fn test_nftoken_create_offer_sell() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // Step 1: mint an NFT to get a token ID.
        let mut mint = NFTokenMint::new(
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
            0,
            None,
            None,
            Some(hex::encode(TEST_NFT_URL).into()),
        );

        let mint_result = submit_and_wait(&mut mint, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to mint NFT");

        let nftoken_id = NFTokenMintResult::try_from(mint_result)
            .expect("Failed to extract NFTokenID")
            .nftoken_id
            .to_string();

        ledger_accept().await;

        // Step 2: create a sell offer for the minted NFT.
        let mut offer = NFTokenCreateOffer::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            Some(vec![NFTokenCreateOfferFlag::TfSellOffer].into()),
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("10000000")), // 10 XRP
            nftoken_id.into(),
            None,
            None,
            None,
        );

        let result = submit_and_wait(&mut offer, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit NFTokenCreateOffer");

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
