//! Wallet funding integration tests.
//!
//! Tests in this file run against Docker standalone rippled (localhost:5005).
//! Faucet-specific tests are in a separate module gated behind the `testnet` feature
//! since they require access to the public XRP Ledger testnet faucet.

#[cfg(all(
    feature = "integration",
    feature = "std",
    feature = "json-rpc",
    feature = "helpers"
))]
mod common;

/// Tests that run against Docker standalone rippled
#[cfg(all(
    feature = "integration",
    feature = "std",
    feature = "json-rpc",
    feature = "helpers"
))]
mod tests {
    use xrpl::{asynch::account::get_xrp_balance, models::XRPAmount};

    use crate::common::{generate_funded_wallet, get_client, with_blockchain_lock};

    #[tokio::test]
    async fn test_wallet_generation_and_funding() {
        with_blockchain_lock(|| async {
            let client = get_client().await;
            let wallet = generate_funded_wallet().await;
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
}
