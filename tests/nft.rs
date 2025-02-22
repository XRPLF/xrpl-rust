#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod common;

#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod tests {
    use crate::common::constants::XRPL_TEST_NET;
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
        let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
        let wallet = generate_faucet_wallet(&client, None, None, None, None)
            .await
            .expect("Failed to generate and fund wallet");

        // Create NFTokenMint transaction
        let mut nft_mint = NFTokenMint::new(
            wallet.classic_address.clone().into(),  // account
            None,                                   // account_txn_id
            None,                                   // fee
            None,                                   // flags
            None,                                   // last_ledger_sequence
            None,                                   // memos
            None,                                   // sequence
            None,                                   // signers
            None,                                   // source_tag
            None,                                   // ticket_sequence
            0,                                      // token_taxon
            None,                                   // issuer
            None,                                   // transfer_fee
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
    }

    #[tokio::test]
    async fn test_create_nft_sell_offer() {
        let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
        let wallet = generate_faucet_wallet(&client, None, None, None, None)
            .await
            .expect("Failed to generate and fund wallet");

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

        let mint_result = submit_and_wait(
            &mut nft_mint,
            &client,
            Some(&wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to mint NFT");

        // Extract NFTokenID from transaction metadata
        let nft_result =
            NFTokenMintResult::try_from(mint_result).expect("Failed to extract NFTokenID");

        // Create sell offer for the minted NFT
        let mut sell_offer = NFTokenCreateOffer::new(
            wallet.classic_address.clone().into(),
            None,                                                   // account_txn_id
            None,                                                   // fee
            Some(vec![NFTokenCreateOfferFlag::TfSellOffer].into()), // flags
            None,                                                   // last_ledger_sequence
            None,                                                   // memos
            None,                                                   // sequence
            None,                                                   // signers
            None,                                                   // source_tag
            None,                                                   // ticket_sequence
            Amount::XRPAmount(XRPAmount::from("10")),               // amount (10 XRP)
            nft_result.nftoken_id.into(),                           // nftoken_id
            None,                                                   // destination
            None,                                                   // expiration
            None,                                                   // owner
        );

        let offer_result = sign_and_submit(&mut sell_offer, &client, &wallet, true, true)
            .await
            .expect("Failed to create NFT sell offer");

        assert!(
            offer_result.engine_result_code >= 0,
            "NFT sell offer creation failed with result: {:?}",
            offer_result
        );
    }
}
