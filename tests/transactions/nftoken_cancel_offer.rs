// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//   (xrpl.js does not have a dedicated NFTokenCancelOffer test file)
//
// Scenarios:
//   - base: mint an NFT, create a sell offer, then cancel it

use std::borrow::Cow;

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{
        results::nftoken::{NFTokenCreateOfferResult, NFTokenMintResult},
        transactions::{
            nftoken_cancel_offer::NFTokenCancelOffer,
            nftoken_create_offer::{NFTokenCreateOffer, NFTokenCreateOfferFlag},
            nftoken_mint::NFTokenMint,
        },
        Amount, XRPAmount,
    },
};

const TEST_NFT_URL: &str = "https://example.com/nft.json";

#[tokio::test]
async fn test_nftoken_cancel_offer_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // Step 1: mint an NFT.
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
        let mut create_offer = NFTokenCreateOffer::new(
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

        let offer_result =
            submit_and_wait(&mut create_offer, client, Some(&wallet), Some(true), Some(true))
                .await
                .expect("Failed to create NFT sell offer");

        let offer_id = NFTokenCreateOfferResult::try_from(offer_result)
            .expect("Failed to extract OfferID")
            .offer_id
            .to_string();

        ledger_accept().await;

        // Step 3: cancel the sell offer.
        // NOTE: Vec<Cow<'a, str>> fields require Cow::Owned(string) instead of .into()
        // so that the inferred lifetime is 'static, satisfying submit_and_wait's
        // `for<'de> Deserialize<'de>` bound.
        let mut cancel = NFTokenCancelOffer::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![Cow::Owned(offer_id)],
        );

        let result = submit_and_wait(&mut cancel, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit NFTokenCancelOffer");

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
