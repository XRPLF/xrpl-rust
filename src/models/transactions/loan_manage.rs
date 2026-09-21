use alloc::{borrow::Cow, format, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    FlagCollection, Model, ValidateCurrencies, XRPAmount, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter, Copy,
)]
#[repr(u32)]
pub enum LoanManageFlag {
    /// Indicates the loan should be defaulted.
    TfLoanDefault = 0x00010000,
    /// Indicates the the loan should be impaired.
    TfLoanImpair = 0x00020000,
    /// Indicates the the loan should be unimpaired.
    TfLoanUnimpair = 0x00040000,
}

/// Manages the state of a Loan ledger entry, including defaulting,
/// impairing, or unimpairing a loan.
/// Only the LoanBroker ledger entry owner can initiate this transaction.
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
pub struct LoanManage<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, LoanManageFlag>,
    /// The ID of the Loan ledger entry to manage.
    #[serde(rename = "LoanID")]
    pub loan_id: Cow<'a, str>,
}

impl Model for LoanManage<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;

        let num_flags = self.common_fields.flags.0.len();
        if num_flags > 1 {
            return Err(XRPLModelException::InvalidValue {
                field: "flags".into(),
                expected: "Only one flag allowed".into(),
                found: format!("{} flags found", num_flags),
            });
        }

        validate_hash256("loan_id", &self.loan_id)?;

        Ok(())
    }
}

impl<'a> Transaction<'a, LoanManageFlag> for LoanManage<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, LoanManageFlag> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanManageFlag> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, LoanManageFlag> for LoanManage<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanManageFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanManage<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<LoanManageFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        loan_id: Cow<'a, str>,
    ) -> LoanManage<'a> {
        LoanManage {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanManage,
                account_txn_id,
                fee,
                flags,
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
            loan_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const SOURCE: &str = "r9LqNeG6qHxLoanManager6T5weJ9mZg";
    const LOAN_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    fn base_tx(
        loan_id: &'static str,
        flags: FlagCollection<LoanManageFlag>,
    ) -> LoanManage<'static> {
        LoanManage {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanManage,
                signing_pub_key: Some("".into()),
                flags,
                ..Default::default()
            },
            loan_id: loan_id.into(),
        }
    }

    #[test]
    fn test_invalid_data_too_long() {
        let tx = LoanManage {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanManage,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanManager6T5weJ9mZg","TransactionType":"LoanManage","Flags":0,"SigningPubKey":"","LoanID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanManage = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_flags() {
        let tx = LoanManage {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanManage,
                signing_pub_key: Some("".into()),
                flags: FlagCollection::new(vec![
                    LoanManageFlag::TfLoanDefault,
                    LoanManageFlag::TfLoanImpair,
                ]),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_id() {
        let tx = LoanManage {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanManage,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDE".into(),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    const VALID_LOAN_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    #[test]
    fn test_new_and_accessors() {
        let mut tx = LoanManage::new(
            SOURCE.into(),
            None,
            Some(XRPAmount::from("10")),
            Some(FlagCollection::new(vec![LoanManageFlag::TfLoanImpair])),
            Some(7108682),
            Some(vec![Memo {
                memo_data: Some("696D70616972".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(100),
            None,
            Some(12345),
            None,
            VALID_LOAN_ID.into(),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(tx.get_transaction_type(), &TransactionType::LoanManage);
        assert_eq!(tx.get_common_fields().account, SOURCE);
        assert_eq!(tx.get_common_fields().sequence, Some(100));
        assert_eq!(tx.loan_id, VALID_LOAN_ID);
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanManage {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanManage,
                ..Default::default()
            },
            loan_id: VALID_LOAN_ID.into(),
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("6D616E6167696E67".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(100));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_invalid_loan_id_empty() {
        assert!(base_tx("", FlagCollection::default()).get_errors().is_err());
        assert!(matches!(
            base_tx("", FlagCollection::default()).get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_id_too_long() {
        // 66 hex chars instead of 64
        let tx = LoanManage {
            loan_id: format!("{}AB", LOAN_ID).into(),
            ..base_tx(LOAN_ID, FlagCollection::default())
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_id_non_hex() {
        // Correct length (64) but starts with a non-hex character
        let tx = base_tx(
            "Z123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD",
            FlagCollection::default(),
        );

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_short_loan_id() {
        // The 34-char value used in test_serde is not a valid hash256
        assert!(base_tx(
            "rDB303FC1C7611B22C09E773B51044F6BE",
            FlagCollection::default()
        )
        .get_errors()
        .is_err());
    }

    #[test]
    fn test_invalid_flags_three() {
        let tx = base_tx(
            LOAN_ID,
            FlagCollection::new(vec![
                LoanManageFlag::TfLoanDefault,
                LoanManageFlag::TfLoanImpair,
                LoanManageFlag::TfLoanUnimpair,
            ]),
        );

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_single_flag_each_variant() {
        for flag in [
            LoanManageFlag::TfLoanDefault,
            LoanManageFlag::TfLoanImpair,
            LoanManageFlag::TfLoanUnimpair,
        ] {
            let tx = base_tx(LOAN_ID, FlagCollection::new(vec![flag]));

            assert!(tx.get_errors().is_ok(), "flag {:?} should be valid", flag);
        }
    }

    #[test]
    fn test_flag_serde_roundtrip() {
        let tx = base_tx(
            LOAN_ID,
            FlagCollection::new(vec![LoanManageFlag::TfLoanImpair]),
        );

        let json = serde_json::to_string(&tx).unwrap();
        let roundtripped: LoanManage = serde_json::from_str(&json).unwrap();

        assert_eq!(tx, roundtripped);
    }

    #[test]
    fn test_new_sets_fields() {
        let tx = LoanManage::new(
            SOURCE.into(),
            None,
            None,
            Some(FlagCollection::new(vec![LoanManageFlag::TfLoanUnimpair])),
            None,
            None,
            None,
            None,
            None,
            None,
            LOAN_ID.into(),
        );

        assert_eq!(tx.get_transaction_type(), &TransactionType::LoanManage);
        assert_eq!(tx.loan_id, LOAN_ID);
        assert_eq!(tx.common_fields.flags.0.len(), 1);
        assert!(tx.get_errors().is_ok());
    }
}
