use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
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
                expected: "Only one flag arrowed".into(),
                found: format!("{} flags found", num_flags),
            });
        }

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
                TransactionType::LoanSet,
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

    const SOURCE: &str = "r9LqNeG6qHxLoanManager6T5weJ9mZg";
    const LOAN_ID: &str = "rDB303FC1C7611B22C09E773B51044F6BE";

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

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanManager6T5weJ9mZg","TransactionType":"LoanManage","Flags":0,"SigningPubKey":"","LoanID":"rDB303FC1C7611B22C09E773B51044F6BE"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(&serde_json::to_string(&tx).unwrap()).unwrap();

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
}
