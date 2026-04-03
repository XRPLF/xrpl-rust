// XLS-65 SingleAssetVault — VaultClawback integration test stub
//
// VaultClawback requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.

use xrpl::models::transactions::vault_clawback::VaultClawback;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::Model;

const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

#[test]
fn test_vault_clawback_serde_roundtrip() {
    let vault_clawback = VaultClawback {
        common_fields: CommonFields {
            account: "rIssuer123".into(),
            transaction_type: TransactionType::VaultClawback,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        holder: "rHolder456".into(),
        amount: Some("500".into()),
    };

    let json_str = serde_json::to_string(&vault_clawback).unwrap();
    let deserialized: VaultClawback = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_clawback, deserialized);
    assert!(vault_clawback.validate().is_ok());
}

#[test]
fn test_vault_clawback_no_amount() {
    let vault_clawback = VaultClawback {
        common_fields: CommonFields {
            account: "rIssuerXRP".into(),
            transaction_type: TransactionType::VaultClawback,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        holder: "rHolderXRP".into(),
        amount: None,
    };

    let json_str = serde_json::to_string(&vault_clawback).unwrap();
    let deserialized: VaultClawback = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_clawback, deserialized);
}

#[test]
fn test_vault_clawback_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_clawback = VaultClawback {
        common_fields: CommonFields {
            account: "rClawBuilder".into(),
            transaction_type: TransactionType::VaultClawback,
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        holder: "rTarget".into(),
        amount: Some("1000".into()),
    }
    .with_fee("12".into())
    .with_sequence(600)
    .with_memo(Memo {
        memo_data: Some("compliance clawback".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_clawback.common_fields.fee.as_ref().unwrap().0, "12");
    assert_eq!(vault_clawback.common_fields.sequence, Some(600));
    assert!(vault_clawback.validate().is_ok());
}
