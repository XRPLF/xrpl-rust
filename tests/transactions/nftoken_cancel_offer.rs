// Scenarios:
//   - base: mint an NFT, create a sell offer, then cancel it

use std::borrow::Cow;

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::{
    asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit},
    models::{
        requests::{account_nfts::AccountNfts, nft_sell_offers::NftSellOffers},
        results,
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

        sign_and_submit(&mut mint, client, &wallet, true, true)
            .await
            .expect("Failed to mint NFT");

        ledger_accept().await;

        // Get the NFT ID from account_nfts
        let nfts_response = client
            .request(
                AccountNfts::new(None, wallet.classic_address.clone().into(), None, None).into(),
            )
            .await
            .expect("Failed to query account_nfts");
        let nfts_result: results::account_nfts::AccountNfts<'_> = nfts_response
            .try_into()
            .expect("Failed to parse account_nfts");

        assert_eq!(nfts_result.nfts.len(), 1, "Expected one NFT after mint");
        let nftoken_id = nfts_result.nfts[0].nft_id.to_string();

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
            nftoken_id.clone().into(),
            None,
            None,
            None,
        );

        sign_and_submit(&mut create_offer, client, &wallet, true, true)
            .await
            .expect("Failed to create NFT sell offer");

        ledger_accept().await;

        // Get the offer ID via nft_sell_offers.
        // NOTE: account_objects has a parsing bug in the SDK (UnexpectedResultType) for NFT-related
        // objects; nft_sell_offers avoids that path entirely.
        let offers_response = client
            .request(NftSellOffers::new(None, nftoken_id.clone().into()).into())
            .await
            .expect("Failed to query nft_sell_offers");
        let offers_result: results::nft_sell_offers::NFTSellOffers<'_> = offers_response
            .try_into()
            .expect("Failed to parse nft_sell_offers");

        assert_eq!(offers_result.offers.len(), 1, "Expected one sell offer");
        let offer_id = offers_result.offers[0].nft_offer_index.to_string();

        // Step 3: cancel the sell offer.
        // NOTE: Vec<Cow<'a, str>> fields require Cow::Owned(string) instead of .into()
        // so that the inferred lifetime is 'static.
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

        test_transaction(&mut cancel, &wallet).await;
    })
    .await;
}
