// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize
//
// Scenarios:
//   - holder_opt_in: a non-issuer authorizes themselves to hold the MPT

use crate::common::{
    create_mptoken_issuance, generate_funded_wallet, test_transaction, with_blockchain_lock,
};
use xrpl::models::transactions::{
    mptoken_authorize::MPTokenAuthorize, CommonFields, TransactionType,
};

#[tokio::test]
async fn test_mptoken_authorize_holder_opt_in() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;

        // Create an issuance first
        let issuance_id = create_mptoken_issuance(&issuer).await;

        // Holder opts in
        let mut tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                fee: Some("10".into()),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            holder: None, // omitted when a holder opts in themselves
        };

        test_transaction(&mut tx, &holder).await;
    })
    .await;
}
