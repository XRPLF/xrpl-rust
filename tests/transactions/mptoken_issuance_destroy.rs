// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancedestroy
//
// Scenarios:
//   - base: issuer destroys an issuance with no outstanding tokens

use crate::common::{
    create_mptoken_issuance, generate_funded_wallet, test_transaction, with_blockchain_lock,
};
use xrpl::models::transactions::{
    mptoken_issuance_destroy::MPTokenIssuanceDestroy, CommonFields, TransactionType,
};

#[tokio::test]
async fn test_mptoken_issuance_destroy_base() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        // Create an issuance first
        let issuance_id = create_mptoken_issuance(&issuer).await;

        // Destroy it (no outstanding tokens, so this should succeed)
        let mut tx = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                fee: Some("10".into()),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
        };

        test_transaction(&mut tx, &issuer).await;
    })
    .await;
}
