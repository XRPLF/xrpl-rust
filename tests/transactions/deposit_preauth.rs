// DepositPreauth transaction integration tests.
//
// Scenarios:
//   1. base: authorize a second account by address (account-based preauth)
//   2. AuthorizeCredentials: credential-based authorization
//   3. UnauthorizeCredentials: revoke credential-based authorization

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock, CREDENTIAL_TYPE_KYC};
use xrpl::models::{
    transactions::{deposit_preauth::DepositPreauth, CommonFields, TransactionType},
    CredentialAuthorization, CredentialAuthorizationFields,
};

// ── 1. Base: account-based preauthorization ───────────────────────────────

#[tokio::test]
async fn test_deposit_preauth_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let authorized = generate_funded_wallet().await;

        let mut tx = DepositPreauth {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: Some(authorized.classic_address.clone().into()),
            ..Default::default()
        };

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

// ── 2. AuthorizeCredentials base case ────────────────────────────────────

#[tokio::test]
async fn test_deposit_preauth_authorize_credentials() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let issuer = generate_funded_wallet().await;

        let creds = vec![CredentialAuthorization::new(
            CredentialAuthorizationFields::new(
                issuer.classic_address.clone().into(),
                CREDENTIAL_TYPE_KYC.into(),
            ),
        )];

        let mut tx = DepositPreauth {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize_credentials: Some(creds),
            ..Default::default()
        };

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

// ── 3. UnauthorizeCredentials base case ──────────────────────────────────

#[tokio::test]
async fn test_deposit_preauth_unauthorize_credentials() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let issuer = generate_funded_wallet().await;

        let cred_type = CREDENTIAL_TYPE_KYC;
        let make_creds = || {
            vec![CredentialAuthorization::new(
                CredentialAuthorizationFields::new(
                    issuer.classic_address.clone().into(),
                    cred_type.into(),
                ),
            )]
        };

        // First: authorize credentials.
        let mut authorize = DepositPreauth {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize_credentials: Some(make_creds()),
            ..Default::default()
        };
        test_transaction(&mut authorize, &wallet).await;

        // Then: revoke the same credential authorization.
        let mut unauthorize = DepositPreauth {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            unauthorize_credentials: Some(make_creds()),
            ..Default::default()
        };
        test_transaction(&mut unauthorize, &wallet).await;
    })
    .await;
}
