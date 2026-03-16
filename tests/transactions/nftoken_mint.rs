// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/nftokenMint.test.ts
//
// Scenarios:
//   - base: mint an NFT with a URI

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::{
    asynch::transaction::submit_and_wait,
    models::transactions::nftoken_mint::NFTokenMint,
};

const TEST_NFT_URL: &str = "https://example.com/nft.json";

#[tokio::test]
async fn test_nftoken_mint_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        // Fresh wallet: NFTokenMint modifies the account's NFToken page objects.
        let wallet = generate_funded_wallet().await;

        let mut tx = NFTokenMint::new(
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
            0,    // transfer_fee
            None, // issuer
            None, // nftoken_taxon (defaults to 0)
            Some(hex::encode(TEST_NFT_URL).into()),
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit NFTokenMint");

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
