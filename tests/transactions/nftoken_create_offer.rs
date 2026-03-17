// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//
// Scenarios:
//   - sell_offer: mint an NFT then create a sell offer for it

use crate::common::{generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock};
use xrpl::{
    asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit},
    models::{
        requests::account_nfts::AccountNfts,
        results,
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

        sign_and_submit(&mut mint, client, &wallet, true, true)
            .await
            .expect("Failed to mint NFT");

        ledger_accept().await;

        // Get the NFT ID from account_nfts
        let nfts_response = client
            .request(
                AccountNfts::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("Failed to query account_nfts");
        let nfts_result: results::account_nfts::AccountNfts<'_> =
            nfts_response.try_into().expect("Failed to parse account_nfts");

        assert_eq!(nfts_result.nfts.len(), 1, "Expected one NFT");
        let nftoken_id = nfts_result.nfts[0].nft_id.to_string();

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

        test_transaction(&mut offer, &wallet).await;
    })
    .await;
}
