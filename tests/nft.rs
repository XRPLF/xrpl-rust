#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod common;

#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod tests {
    use crate::common::{constants::XRPL_TEST_NET, get_client, get_wallet, with_blockchain_lock};
    use url::Url;
    use xrpl::{
        asynch::{
            clients::AsyncJsonRpcClient,
            transaction::{sign_and_submit, submit_and_wait},
            wallet::generate_faucet_wallet,
        },
        models::{
            results::nftoken::NFTokenMintResult,
            transactions::{
                nftoken_create_offer::{NFTokenCreateOffer, NFTokenCreateOfferFlag},
                nftoken_mint::NFTokenMint,
            },
            Amount, XRPAmount,
        },
    };

    const TEST_NFT_URL: &'static str = "https://example.com/nft.json";

    #[tokio::test]
    async fn test_mint_nft() {
        with_blockchain_lock(|| async {
            let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
            let wallet = generate_faucet_wallet(&client, None, None, None, None)
                .await
                .expect("Failed to generate and fund wallet");

            // Create NFTokenMint transaction
            let mut nft_mint = NFTokenMint::new(
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
                0,
                None,
                None,
                Some(hex::encode(TEST_NFT_URL).into()), // URI as hex
            );

            // Sign and submit the transaction
            let result = sign_and_submit(&mut nft_mint, &client, &wallet, true, true)
                .await
                .expect("Failed to submit NFTokenMint transaction");

            assert!(
                result.engine_result_code >= 0,
                "NFT minting failed with result: {:?}",
                result
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_nft_sell_offer() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;

            // First mint an NFT
            let mut nft_mint = NFTokenMint::new(
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

            // Submit the NFTokenMint transaction and wait for validation
            let mint_result =
                submit_and_wait(&mut nft_mint, client, Some(wallet), Some(true), Some(true))
                    .await
                    .expect("Failed to mint NFT");

            // Extract NFTokenID from transaction metadata
            let nft_result =
                NFTokenMintResult::try_from(mint_result).expect("Failed to extract NFTokenID");
            let nftoken_id = nft_result.nftoken_id.to_string();

            // Create a new transaction for the sell offer
            let mut sell_offer = NFTokenCreateOffer::new(
                wallet.classic_address.clone().into(),
                None,
                None,
                Some(vec![NFTokenCreateOfferFlag::TfSellOffer].into()), // flags
                None,
                None,
                None,
                None,
                None,
                None,
                Amount::XRPAmount(XRPAmount::from("10")), // amount (10 XRP)
                nftoken_id.into(),                        // nftoken_id
                None,
                None,
                None,
            );

            let offer_result = sign_and_submit(&mut sell_offer, client, wallet, true, true)
                .await
                .expect("Failed to create NFT sell offer");

            assert!(
                offer_result.engine_result_code >= 0,
                "NFT sell offer creation failed with result: {:?}",
                offer_result
            );
        })
        .await;
    }
}
