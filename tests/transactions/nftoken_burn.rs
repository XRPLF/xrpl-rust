// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//   (xrpl.js does not have a dedicated NFTokenBurn test file)
//
// Scenarios:
//   - base: mint an NFT then burn it

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::{
    asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit},
    models::{
        requests::account_nfts::AccountNfts,
        results,
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

        test_transaction(&mut burn, &wallet).await;
    })
    .await;
}
