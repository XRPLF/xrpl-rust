// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/clawback
//
// Scenarios:
//   - ica_clawback: issuer claws back IOU from a holder (requires featureClawback)
//   - mpt_clawback: issuer claws back MPT from an authorized holder (requires featureMPTokensV1)
//
// Mirrors xrpl.js packages/xrpl/test/integration/transactions/clawback.test.ts

use crate::common::{
    generate_funded_wallet, get_client, get_ledger_close_time, ledger_accept, test_transaction,
    wait_for_ledger_close_time, with_blockchain_lock,
};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::{
    transactions::{
        account_set::{AccountSet, AccountSetFlag},
        clawback::Clawback,
        mptoken_authorize::MPTokenAuthorize,
        mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
        payment::Payment,
        trust_set::TrustSet,
        CommonFields, TransactionType,
    },
    Amount, IssuedCurrencyAmount, MPTAmount,
};

/// ICA Clawback: issuer enables clawback, holder receives IOU, issuer claws back.
/// Requires featureClawback amendment to be enabled in the test environment.
#[tokio::test]
async fn test_clawback_ica() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;

        // 1. Issuer enables AllowTrustLineClawback — must be set before any trust lines
        let mut acct_set_clawback = AccountSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            set_flag: Some(AccountSetFlag::AsfAllowTrustLineClawback),
            ..Default::default()
        };
        test_transaction(&mut acct_set_clawback, &issuer).await;

        // 2. Holder creates trust line for USD from issuer
        let mut trust = TrustSet {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                issuer.classic_address.clone().into(),
                "10000".into(),
            ),
            quality_in: None,
            quality_out: None,
        };
        test_transaction(&mut trust, &holder).await;

        // 3. Issuer pays 1000 USD to holder
        let mut payment = Payment {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                issuer.classic_address.clone().into(),
                "1000".into(),
            )),
            destination: holder.classic_address.clone().into(),
            ..Default::default()
        };
        test_transaction(&mut payment, &issuer).await;

        // 4. Issuer claws back 500 USD from holder
        // For ICA clawback: Amount.issuer = HOLDER address (the account being clawed from)
        let mut clawback = Clawback {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                holder.classic_address.clone().into(),
                "500".into(),
            )),
            holder: None, // ICA clawback: Holder field absent
        };
        test_transaction(&mut clawback, &issuer).await;
    })
    .await;
}

/// MPT Clawback: issuer creates MPT with clawback flag, holder receives MPT,
/// issuer claws back using the Holder field.
/// Requires featureMPTokensV1 amendment to be enabled in the test environment.
#[tokio::test]
async fn test_clawback_mpt() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let client = get_client().await;

        // 1. Create MPT issuance with transfer + clawback enabled
        let mut create_tx = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                flags: vec![
                    MPTokenIssuanceCreateFlag::TfMPTCanTransfer,
                    MPTokenIssuanceCreateFlag::TfMPTCanClawback,
                ]
                .into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = sign_and_submit(&mut create_tx, client, &issuer, true, true)
            .await
            .expect("MPTokenIssuanceCreate sign_and_submit failed");
        assert_eq!(
            result.engine_result, "tesSUCCESS",
            "MPTokenIssuanceCreate failed: {}",
            result.engine_result
        );
        let pre_close = get_ledger_close_time().await;
        ledger_accept().await;
        wait_for_ledger_close_time(pre_close + 1).await;

        // Derive MPTokenIssuanceID from autofilled sequence + account
        let sequence = result.tx_json["Sequence"]
            .as_u64()
            .expect("Sequence missing") as u32;
        let account_id = xrpl::core::addresscodec::decode_classic_address(&issuer.classic_address)
            .expect("decode classic address failed");
        let mut id_bytes = Vec::with_capacity(24);
        id_bytes.extend_from_slice(&sequence.to_be_bytes());
        id_bytes.extend_from_slice(&account_id);
        let issuance_id = hex::encode_upper(&id_bytes);

        // 2. Holder opts in
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

        // 3. Issuer pays 1000 MPT to holder
        // (re-use create_mptoken_issuance helper for the base issuance ID used in payment)
        let transferable_id = {
            let mut pay_tx = Payment {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::Payment,
                    ..Default::default()
                },
                amount: Amount::MPTAmount(MPTAmount {
                    value: "1000".into(),
                    mpt_issuance_id: issuance_id.clone().into(),
                }),
                destination: holder.classic_address.clone().into(),
                ..Default::default()
            };
            test_transaction(&mut pay_tx, &issuer).await;
            issuance_id.clone()
        };

        // 4. Issuer claws back 500 MPT from holder using Holder field
        let mut clawback = Clawback {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::MPTAmount(MPTAmount {
                value: "500".into(),
                mpt_issuance_id: transferable_id.into(),
            }),
            holder: Some(holder.classic_address.clone().into()),
        };
        test_transaction(&mut clawback, &issuer).await;
    })
    .await;
}
