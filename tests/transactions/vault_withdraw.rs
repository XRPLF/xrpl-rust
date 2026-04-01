// XLS-65 SingleAssetVault — VaultWithdraw integration test stub
//
// VaultWithdraw requires an XLS-65-enabled rippled node. These tests validate
// that the transaction type can be constructed and serialized correctly.

use xrpl::models::transactions::vault_withdraw::VaultWithdraw;
use xrpl::models::transactions::{CommonFields, Memo, TransactionType};
use xrpl::models::{Amount, IssuedCurrencyAmount, Model, XRPAmount};

const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

#[test]
fn test_vault_withdraw_serde_roundtrip_xrp() {
    let vault_withdraw = VaultWithdraw {
        common_fields: CommonFields {
            account: "rWithdrawer123".into(),
            transaction_type: TransactionType::VaultWithdraw,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::XRPAmount(XRPAmount::from("5000000")),
    };

    let json_str = serde_json::to_string(&vault_withdraw).unwrap();
    let deserialized: VaultWithdraw = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_withdraw, deserialized);
}

#[test]
fn test_vault_withdraw_serde_roundtrip_issued() {
    let vault_withdraw = VaultWithdraw {
        common_fields: CommonFields {
            account: "rWithdrawICA".into(),
            transaction_type: TransactionType::VaultWithdraw,
            signing_pub_key: Some("".into()),
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            "rIssuer789".into(),
            "500".into(),
        )),
    };

    let json_str = serde_json::to_string(&vault_withdraw).unwrap();
    let deserialized: VaultWithdraw = serde_json::from_str(&json_str).unwrap();
    assert_eq!(vault_withdraw, deserialized);
    assert!(vault_withdraw.validate().is_ok());
}

#[test]
fn test_vault_withdraw_builder_pattern() {
    use xrpl::models::transactions::CommonTransactionBuilder;

    let vault_withdraw = VaultWithdraw {
        common_fields: CommonFields {
            account: "rWdBuilder".into(),
            transaction_type: TransactionType::VaultWithdraw,
            ..Default::default()
        },
        vault_id: VAULT_ID.into(),
        amount: Amount::XRPAmount(XRPAmount::from("1000000")),
    }
    .with_fee("12".into())
    .with_sequence(500)
    .with_memo(Memo {
        memo_data: Some("vault withdrawal".into()),
        memo_format: None,
        memo_type: Some("text".into()),
    });

    assert_eq!(vault_withdraw.common_fields.fee.as_ref().unwrap().0, "12");
    assert_eq!(vault_withdraw.common_fields.sequence, Some(500));
    assert!(vault_withdraw.validate().is_ok());
}
