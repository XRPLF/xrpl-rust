// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate
//
// Scenarios:
//   - base: create an MPToken issuance with defaults
//   - with_metadata: create with metadata and asset_scale

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::{
    mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
    CommonFields, TransactionType,
};

#[tokio::test]
async fn test_mptoken_issuance_create_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        };

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

#[tokio::test]
async fn test_mptoken_issuance_create_with_metadata() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            transfer_fee: Some(314),
            mptoken_metadata: Some("CAFEBABE".into()),
        };

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}
