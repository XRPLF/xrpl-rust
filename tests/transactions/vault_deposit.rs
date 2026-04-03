// XLS-65 SingleAssetVault — VaultDeposit integration test stub
//
// VaultDeposit requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.

use xrpl::models::transactions::vault_deposit::VaultDeposit;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::{Amount, IssuedCurrencyAmount, Model, XRPAmount};

const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

#[test]
fn test_vault_deposit_serde_roundtrip_xrp() {
    let vault_deposit = VaultDeposit {
        common_fields: CommonFields {
            account: "rDepositor123".into(),
            transaction_type: TransactionType::VaultDeposit,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::XRPAmount(XRPAmount::from("5000000")),
    };

    let json_str = serde_json::to_string(&vault_deposit).unwrap();
    let deserialized: VaultDeposit = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_deposit, deserialized);
}

#[test]
fn test_vault_deposit_serde_roundtrip_issued() {
    let vault_deposit = VaultDeposit {
        common_fields: CommonFields {
            account: "rDepositorICA".into(),
            transaction_type: TransactionType::VaultDeposit,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            "rIssuer789".into(),
            "1000".into(),
        )),
    };

    let json_str = serde_json::to_string(&vault_deposit).unwrap();
    let deserialized: VaultDeposit = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_deposit, deserialized);
    assert!(vault_deposit.validate().is_ok());
}

#[test]
fn test_vault_deposit_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_deposit = VaultDeposit {
        common_fields: CommonFields {
            account: "rDepBuilder".into(),
            transaction_type: TransactionType::VaultDeposit,
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::XRPAmount(XRPAmount::from("1000000")),
    }
    .with_fee("12".into())
    .with_sequence(400)
    .with_memo(Memo {
        memo_data: Some("vault deposit".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_deposit.common_fields.fee.as_ref().unwrap().0, "12");
    assert_eq!(vault_deposit.common_fields.sequence, Some(400));
    assert!(vault_deposit.validate().is_ok());
}
