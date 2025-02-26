#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod common;

#[cfg(all(feature = "std", feature = "json-rpc", feature = "helpers"))]
mod tests {
    use url::Url;
    use xrpl::{
        asynch::{
            account::get_xrp_balance, clients::AsyncJsonRpcClient, wallet::generate_faucet_wallet,
        },
        models::XRPAmount,
    };

    use crate::common::{constants::XRPL_TEST_NET, get_client, get_wallet, with_blockchain_lock};

    #[tokio::test]
    async fn test_wallet_generation_and_funding() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = get_wallet().await;
            let address: String = wallet.classic_address.clone();

            // Verify wallet properties
            assert!(
                !wallet.classic_address.is_empty(),
                "Wallet should have a classic address"
            );
            assert!(
                !wallet.public_key.is_empty(),
                "Wallet should have a public key"
            );
            assert!(
                !wallet.private_key.is_empty(),
                "Wallet should have a private key"
            );

            // Verify the wallet has been funded
            let balance: XRPAmount = get_xrp_balance(address.into(), client, None)
                .await
                .expect("Failed to get wallet balance");

            assert!(
                balance > XRPAmount::from("0"),
                "Wallet should have a positive balance"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_wallet_with_custom_faucet() {
        let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());
        let custom_faucet = Url::parse("https://faucet.altnet.rippletest.net/accounts").unwrap();

        let wallet = generate_faucet_wallet(&client, None, Some(custom_faucet), None, None)
            .await
            .expect("Failed to generate and fund wallet with custom faucet");

        let address: String = wallet.classic_address.clone();

        let balance: XRPAmount = get_xrp_balance(address.into(), &client, None)
            .await
            .expect("Failed to get wallet balance");

        assert!(
            balance > XRPAmount::from("0"),
            "Wallet should have a positive balance"
        );
    }

    #[tokio::test]
    async fn test_wallet_reuse() {
        let client = AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap());

        // Create an initial wallet
        let existing_wallet = xrpl::wallet::Wallet::create(None).expect("Failed to create wallet");
        let existing_address = existing_wallet.classic_address.clone();

        // Fund the existing wallet
        let funded_wallet =
            generate_faucet_wallet(&client, Some(existing_wallet), None, None, None)
                .await
                .expect("Failed to fund existing wallet");
        let funded_address = funded_wallet.classic_address.clone();

        // Verify it's the same wallet
        assert_eq!(
            existing_address, funded_address,
            "Funded wallet should match existing wallet"
        );

        let balance: XRPAmount = get_xrp_balance(funded_address.into(), &client, None)
            .await
            .expect("Failed to get wallet balance");

        assert!(
            balance > XRPAmount::from("0"),
            "Wallet should have a positive balance"
        );
    }
}
