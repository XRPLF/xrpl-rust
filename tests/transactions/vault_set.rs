// XLS-65 SingleAssetVault — VaultSet integration test stub
//
// VaultSet requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.

use xrpl::models::transactions::vault_set::VaultSet;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::Model;

const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

#[test]
fn test_vault_set_serde_roundtrip() {
    let vault_set = VaultSet {
        common_fields: CommonFields {
            account: "rVaultOwner123".into(),
            transaction_type: TransactionType::VaultSet,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        data: Some("48656C6C6F".into()),
        assets_maximum: None,
        domain_id: None,
    };

    let json_str = serde_json::to_string(&vault_set).unwrap();
    let deserialized: VaultSet = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_set, deserialized);
    assert!(vault_set.validate().is_ok());
}

#[test]
fn test_vault_set_all_optional_fields() {
    let vault_set = VaultSet {
        common_fields: CommonFields {
            account: "rVaultOwnerFull".into(),
            transaction_type: TransactionType::VaultSet,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        data: Some("48656C6C6F".into()),
        assets_maximum: Some("2000000000".into()),
        domain_id: Some("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into()),
    };

    let json_str = serde_json::to_string(&vault_set).unwrap();
    let deserialized: VaultSet = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_set, deserialized);
}

#[test]
fn test_vault_set_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_set = VaultSet {
        common_fields: CommonFields {
            account: "rSetBuilder".into(),
            transaction_type: TransactionType::VaultSet,
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        ..Default::default()
    }
    .with_fee("12".into())
    .with_sequence(700)
    .with_memo(Memo {
        memo_data: Some("updating vault".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_set.common_fields.fee.as_ref().unwrap().0, "12");
    assert_eq!(vault_set.common_fields.sequence, Some(700));
    assert!(vault_set.validate().is_ok());
}
