// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset
//
// Scenarios:
//   - lock_issuance: issuer locks all tokens at the issuance level

use crate::common::{
    create_mptoken_issuance, generate_funded_wallet, test_transaction, with_blockchain_lock,
};
use xrpl::models::transactions::{
    mptoken_issuance_set::{MPTokenIssuanceSet, MPTokenIssuanceSetFlag},
    CommonFields, TransactionType,
};

#[tokio::test]
async fn test_mptoken_issuance_set_lock() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        // Create an issuance first
        let issuance_id = create_mptoken_issuance(&issuer).await;

        // Lock the issuance
        let mut tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            holder: None, // lock the entire issuance
        };

        test_transaction(&mut tx, &issuer).await;
    })
    .await;
}
