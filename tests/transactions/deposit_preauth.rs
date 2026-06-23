// DepositPreauth transaction integration tests.
//
// Scenarios:
//   1. base: authorize a second account by address (account-based preauth)
//   2. AuthorizeCredentials: credential-based authorization (requires Credentials amendment)
//   3. UnauthorizeCredentials: revoke credential-based authorization
//
// Note: scenarios 2 and 3 are marked #[ignore] because they require the
// Credentials amendment enabled in the standalone rippled node. Once CI
// configures the amendment, remove the #[ignore] attributes.

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::{
    credential_authorization::{CredentialAuthorization, CredentialAuthorizationFields},
    transactions::deposit_preauth::DepositPreauth,
};

// ── 1. Base: account-based preauthorization ───────────────────────────────

#[tokio::test]
async fn test_deposit_preauth_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let authorized = generate_funded_wallet().await;

        let mut tx = DepositPreauth::new(
            wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Some(authorized.classic_address.clone().into()), // authorize
            None,
            None, // unauthorize
            None,
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

// ── 2. AuthorizeCredentials base case ────────────────────────────────────
//
// Requires: standalone rippled with Credentials amendment enabled.
// Remove #[ignore] once CI has the amendment configured
// (outbound to testnet.xrpl-labs.com / faucet.altnet.rippletest.net also needed).

#[tokio::test]
#[ignore = "requires Credentials amendment enabled in standalone rippled"]
async fn test_deposit_preauth_authorize_credentials() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let issuer = generate_funded_wallet().await;

        let creds = vec![CredentialAuthorization::new(CredentialAuthorizationFields::new(
            issuer.classic_address.clone().into(),
            "4B5943".into(), // hex "KYC"
        ))];

        let mut tx = DepositPreauth::new(
            wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            None,            // authorize (account-based) — None
            Some(creds),     // authorize_credentials
            None,
            None,
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

// ── 3. UnauthorizeCredentials base case ──────────────────────────────────
//
// Requires: standalone rippled with Credentials amendment enabled.

#[tokio::test]
#[ignore = "requires Credentials amendment enabled in standalone rippled"]
async fn test_deposit_preauth_unauthorize_credentials() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let issuer = generate_funded_wallet().await;

        let cred_type = "4B5943"; // hex "KYC"
        let make_creds = || {
            vec![CredentialAuthorization::new(CredentialAuthorizationFields::new(
                issuer.classic_address.clone().into(),
                cred_type.into(),
            ))]
        };

        // First: authorize credentials.
        let mut authorize = DepositPreauth::new(
            wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            None,
            Some(make_creds()),
            None,
            None,
        );
        test_transaction(&mut authorize, &wallet).await;

        // Then: revoke the same credential authorization.
        let mut unauthorize = DepositPreauth::new(
            wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            None,
            None,
            None,
            Some(make_creds()), // unauthorize_credentials
        );
        test_transaction(&mut unauthorize, &wallet).await;
    })
    .await;
}
