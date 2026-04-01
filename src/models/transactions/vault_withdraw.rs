use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPLModelResult};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Withdraw assets from a vault on the XRP Ledger (XLS-65).
///
/// The withdrawer burns share tokens (MPTokens) in exchange for the
/// proportional share of the vault's assets.
///
/// See VaultWithdraw transaction:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct VaultWithdraw<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the vault to withdraw from (256-bit hex string).
    #[serde(rename = "VaultID")]
    pub vault_id: Cow<'a, str>,
    /// The amount of the asset to withdraw from the vault.
    pub amount: Amount<'a>,
}

impl Model for VaultWithdraw<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for VaultWithdraw<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for VaultWithdraw<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> VaultWithdraw<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        vault_id: Cow<'a, str>,
        amount: Amount<'a>,
    ) -> VaultWithdraw<'a> {
        VaultWithdraw {
            common_fields: CommonFields::new(
                account,
                TransactionType::VaultWithdraw,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            vault_id,
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IssuedCurrencyAmount, XRPAmount};

    const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

    #[test]
    fn test_serde() {
        let vault_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rWithdrawer123".into(),
                transaction_type: TransactionType::VaultWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
        };

        let json_str = r#"{"Account":"rWithdrawer123","TransactionType":"VaultWithdraw","Flags":0,"SigningPubKey":"","VaultID":"A0000000000000000000000000000000000000000000000000000000DEADBEEF","Amount":"1000000"}"#;

        // Serialize
        let serialized = serde_json::to_string(&vault_withdraw).unwrap();
        assert_eq!(
            serde_json::to_value(&serialized).unwrap(),
            serde_json::to_value(json_str).unwrap()
        );

        // Deserialize
        let deserialized: VaultWithdraw = serde_json::from_str(json_str).unwrap();
        assert_eq!(vault_withdraw, deserialized);
    }

    #[test]
    fn test_serde_issued_currency() {
        let vault_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rWithdrawICA456".into(),
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

        let serialized = serde_json::to_string(&vault_withdraw).unwrap();
        let deserialized: VaultWithdraw = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault_withdraw, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let vault_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rWithdrawer123".into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("withdrawing from vault".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(vault_withdraw.vault_id, VAULT_ID);
        assert_eq!(vault_withdraw.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_withdraw.common_fields.sequence, Some(100));
        assert_eq!(
            vault_withdraw.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(vault_withdraw.common_fields.source_tag, Some(12345));
        assert_eq!(
            vault_withdraw.common_fields.memos.as_ref().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_default() {
        let vault_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rWithdrawer789".into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("5000000")),
        };

        assert_eq!(vault_withdraw.common_fields.account, "rWithdrawer789");
        assert_eq!(
            vault_withdraw.common_fields.transaction_type,
            TransactionType::VaultWithdraw
        );
        assert_eq!(vault_withdraw.vault_id, VAULT_ID);
        assert!(vault_withdraw.common_fields.fee.is_none());
        assert!(vault_withdraw.common_fields.sequence.is_none());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rTicketWithdrawer111".into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("2000000")),
        }
        .with_ticket_sequence(54321)
        .with_fee("12".into());

        assert_eq!(ticket_withdraw.common_fields.ticket_sequence, Some(54321));
        assert!(ticket_withdraw.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rMultiMemoWithdrawer222".into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rUSDIssuer333".into(),
                "250".into(),
            )),
        }
        .with_memo(Memo {
            memo_data: Some("partial withdrawal".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("rebalancing portfolio".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_withdraw
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_withdraw.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_new_constructor() {
        let vault_withdraw = VaultWithdraw::new(
            "rNewWithdrawer444".into(),
            None,
            Some("12".into()),
            Some(7108682),
            None,
            Some(100),
            None,
            None,
            None,
            VAULT_ID.into(),
            Amount::XRPAmount(XRPAmount::from("10000000")),
        );

        assert_eq!(vault_withdraw.common_fields.account, "rNewWithdrawer444");
        assert_eq!(
            vault_withdraw.common_fields.transaction_type,
            TransactionType::VaultWithdraw
        );
        assert_eq!(vault_withdraw.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_withdraw.vault_id, VAULT_ID);
    }

    #[test]
    fn test_validate() {
        let vault_withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: "rValidateWithdrawer555".into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
        }
        .with_fee("12".into())
        .with_sequence(300);

        assert!(vault_withdraw.validate().is_ok());
    }
}
