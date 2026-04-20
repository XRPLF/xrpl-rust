// XLS-65 SingleAssetVault — VaultDelete integration test stub
//
// VaultDelete requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.

use xrpl::models::transactions::vault_delete::VaultDelete;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::Model;

const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

#[test]
fn test_vault_delete_serde_roundtrip() {
    let vault_delete = VaultDelete {
        common_fields: CommonFields {
            account: "rVaultOwner123".into(),
            transaction_type: TransactionType::VaultDelete,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
    };

    let json_str = serde_json::to_string(&vault_delete).unwrap();
    let deserialized: VaultDelete = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_delete, deserialized);
}

#[test]
fn test_vault_delete_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_delete = VaultDelete {
        common_fields: CommonFields {
            account: "rVaultDelBuilder".into(),
            transaction_type: TransactionType::VaultDelete,
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
    }
    .with_fee("12".into())
    .with_sequence(300)
    .with_memo(Memo {
        memo_data: Some("vault deletion".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_delete.common_fields.fee.as_ref().unwrap().0, "12");
    assert_eq!(vault_delete.common_fields.sequence, Some(300));
    assert!(vault_delete.validate().is_ok());
}
