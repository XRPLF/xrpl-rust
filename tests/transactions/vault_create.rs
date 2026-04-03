// XLS-65 SingleAssetVault — VaultCreate integration test stub
//
// VaultCreate requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.
// Live submission tests will be enabled once XLS-65 is available on devnet.

use xrpl::models::transactions::vault_create::VaultCreate;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::{Currency, IssuedCurrency, Model, XRP};

#[test]
fn test_vault_create_serde_roundtrip() {
    let vault_create = VaultCreate {
        common_fields: CommonFields {
            account: "rVaultCreator123".into(),
            transaction_type: TransactionType::VaultCreate,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
        data: None,
        assets_maximum: None,
        mptoken_metadata: None,
        domain_id: None,
        withdrawal_policy: None,
        scale: None,
    };

    let json_str = serde_json::to_string(&vault_create).unwrap();
    let deserialized: VaultCreate = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_create, deserialized);
}

#[test]
fn test_vault_create_with_all_optional_fields() {
    let vault_create = VaultCreate {
        common_fields: CommonFields {
            account: "rVaultCreatorFull".into(),
            transaction_type: TransactionType::VaultCreate,
            fee: Some("12".into()),
            sequence: Some(100),
            ..Default::default()
        },
        asset: Currency::XRP(XRP::new()),
        data: Some("48656C6C6F".into()),
        assets_maximum: Some("1000000000".into()),
        mptoken_metadata: Some("ABCDEF".into()),
        domain_id: Some("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into()),
        withdrawal_policy: Some(1),
        scale: Some(6),
    };

    assert!(vault_create.validate().is_ok());
    let json_str = serde_json::to_string(&vault_create).unwrap();
    let deserialized: VaultCreate = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_create, deserialized);
}

#[test]
fn test_vault_create_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_create = VaultCreate {
        common_fields: CommonFields {
            account: "rVaultBuilder".into(),
            transaction_type: TransactionType::VaultCreate,
            ..Default::default()
        },
        asset: Currency::IssuedCurrency(IssuedCurrency::new("EUR".into(), "rEURIssuer".into())),
        ..Default::default()
    }
    .with_fee("15".into())
    .with_sequence(200)
    .with_memo(Memo {
        memo_data: Some("vault creation".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_create.common_fields.fee.as_ref().unwrap().0, "15");
    assert_eq!(vault_create.common_fields.sequence, Some(200));
    assert!(vault_create.common_fields.memos.is_some());
}
