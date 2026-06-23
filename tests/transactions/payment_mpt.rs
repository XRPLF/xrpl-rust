// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/payment
//
// Scenarios:
//   - mpt_payment: MPTAmount Payment between two authorized accounts
//
// Validates that MPTAmount serializes/deserializes correctly through the wire
// format when submitted to a live rippled node.

use crate::common::{
    create_transferable_mptoken_issuance, generate_funded_wallet, test_transaction,
    with_blockchain_lock,
};
use xrpl::models::{
    transactions::{
        mptoken_authorize::MPTokenAuthorize, payment::Payment, CommonFields, TransactionType,
    },
    Amount, MPTAmount,
};

#[tokio::test]
async fn test_mpt_payment() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;

        // 1. Create MPT issuance with TfMPTCanTransfer
        let issuance_id = create_transferable_mptoken_issuance(&issuer).await;

        // 2. Holder opts in to hold the MPT
        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        // 3. Issuer pays 1000 MPT to holder — validates MPTAmount wire serialization
        let mut payment = Payment {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::MPTAmount(MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: issuance_id.into(),
            }),
            destination: holder.classic_address.clone().into(),
            ..Default::default()
        };
        test_transaction(&mut payment, &issuer).await;
        // test_transaction asserts tesSUCCESS — MPTAmount wire round-trip is verified.
    })
    .await;
}
