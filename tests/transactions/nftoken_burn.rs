// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//   (xrpl.js does not have a dedicated NFTokenBurn test file)
//
// Scenarios:
//   - base: mint an NFT then burn it

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::{
        results::nftoken::NFTokenMintResult,
        transactions::{nftoken_burn::NFTokenBurn, nftoken_mint::NFTokenMint},
    },
};

const TEST_NFT_URL: &str = "https://example.com/nft.json";

#[tokio::test]
async fn test_nftoken_burn_base() {
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

        // Step 2: burn the minted NFT.
        let mut burn = NFTokenBurn::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            nftoken_id.into(),
            None, // owner: None because the burner is the issuer/owner
        );

        let result = submit_and_wait(&mut burn, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit NFTokenBurn");

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
