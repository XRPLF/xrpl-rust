use alloc::{borrow::Cow, format, string::ToString, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, Model, ValidateCurrencies, XRPAmount, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter, Copy,
)]
#[repr(u32)]
pub enum LoanPayFlag {
    /// Indicates that the remaining payment amount should
    /// be treated as an overpayment.
    TfLoanOverpayment = 0x00010000,
    /// Indicates that the borrower is making a full early repayment.
    TfLoanFullPayment = 0x00020000,
    /// Indicates that the borrower is making a late loan payment.
    TfLoanLatePayment = 0x00040000,
}

/// Makes a payment on an active loan.
/// Only the borrower on the loan can make payments, and
/// payments must meet the minimum amount required for that period.
/// A loan payment has four types: Regular Payment, Late Payment, Early Full Payment and Overpayment.
/// `<https://xrpl.org/docs/references/protocol/transactions/types/loanpay>`
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
pub struct LoanPay<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, LoanPayFlag>,
    /// The ID of the Loan ledger entry to repay.
    #[serde(rename = "LoanID")]
    pub loan_id: Cow<'a, str>,
    /// The amount to pay toward the loan.
    pub amount: Amount<'a>,
}

impl Model for LoanPay<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;

        validate_hash256("loan_id", &self.loan_id)?;

        let num_flags = self.common_fields.flags.0.len();
        if num_flags > 1 {
            return Err(XRPLModelException::InvalidValue {
                field: "flags".into(),
                expected: "Only one flag allowed".into(),
                found: format!("{} flags found", num_flags),
            });
        }

        let value = match &self.amount {
            Amount::MPTAmount(amount) => amount.value.as_ref(),
            Amount::IssuedCurrencyAmount(amount) => amount.value.as_ref(),
            Amount::XRPAmount(amount) => amount.0.as_ref(),
        };

        let parsed = bigdecimal::BigDecimal::from_str(value).map_err(|_| {
            XRPLModelException::InvalidValueFormat {
                field: "amount".to_string(),
                format: "a valid decimal number".to_string(),
                found: value.to_string(),
            }
        })?;

        if parsed <= 0 {
            return Err(XRPLModelException::InvalidValue {
                field: "amount".to_string(),
                expected: "a positive amount".to_string(),
                found: value.to_string(),
            });
        }

        Ok(())
    }
}

impl<'a> Transaction<'a, LoanPayFlag> for LoanPay<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, LoanPayFlag> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanPayFlag> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, LoanPayFlag> for LoanPay<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, LoanPayFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanPay<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<LoanPayFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        loan_id: Cow<'a, str>,
        amount: Amount<'a>,
    ) -> LoanPay<'a> {
        LoanPay {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanPay,
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
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const SOURCE: &str = "r9LqNeG6qHxLoanPayer6T5weJ9mZg";
    const LOAN_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    #[test]
    fn test_serde() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::XRPAmount(XRPAmount("1000".into())),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanPayer6T5weJ9mZg","TransactionType":"LoanPay","Flags":0,"SigningPubKey":"","LoanID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":"1000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanPay = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_flags() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                flags: FlagCollection::new(vec![
                    LoanPayFlag::TfLoanFullPayment,
                    LoanPayFlag::TfLoanLatePayment,
                ]),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::XRPAmount(XRPAmount("1000".into())),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_amount() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                flags: FlagCollection::default(),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::XRPAmount(XRPAmount("0".into())),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_id() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                flags: FlagCollection::default(),
                ..Default::default()
            },
            loan_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDE".into(),
            amount: Amount::XRPAmount(XRPAmount("1000".into())),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    const MPT_ISSUANCE_ID: &str = "00000012E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1E1A1";

    #[test]
    fn test_new_and_accessors() {
        let mut tx = LoanPay::new(
            SOURCE.into(),
            None,
            Some(XRPAmount::from("12")),
            Some(FlagCollection::new(vec![LoanPayFlag::TfLoanFullPayment])),
            Some(7108682),
            Some(vec![Memo {
                memo_data: Some("7061796D656E74".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(8),
            None,
            Some(12345),
            None,
            LOAN_ID.into(),
            Amount::XRPAmount(XRPAmount("1000".into())),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(tx.get_transaction_type(), &TransactionType::LoanPay);
        assert_eq!(tx.get_common_fields().account, SOURCE);
        assert_eq!(tx.get_common_fields().sequence, Some(8));
        assert_eq!(tx.loan_id, LOAN_ID);
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::XRPAmount(XRPAmount("1000".into())),
        }
        .with_fee("12".into())
        .with_sequence(8)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("7061796D656E74".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(8));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
        assert!(tx.get_errors().is_ok());
    }

    /// The payment amount is read from whichever `Amount` variant is supplied, so
    /// each variant has to be accepted by `get_errors`.
    #[test]
    fn test_valid_issued_currency_and_mpt_amounts() {
        let base = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::XRPAmount(XRPAmount("1000".into())),
        };

        let issued = LoanPay {
            amount: Amount::IssuedCurrencyAmount(crate::models::IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "1000".into(),
            }),
            ..base.clone()
        };
        assert!(issued.get_errors().is_ok());

        let mpt = LoanPay {
            amount: Amount::MPTAmount(crate::models::MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: MPT_ISSUANCE_ID.into(),
            }),
            ..base
        };
        assert!(mpt.get_errors().is_ok());
    }

    /// `IssuedCurrencyAmount` validates its value with `f64::parse`, which accepts
    /// `NaN` and the infinities. `BigDecimal` does not, so the amount parse in
    /// `get_errors` still has to report a format error for those values.
    #[test]
    fn test_unparsable_amount() {
        let tx = LoanPay {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanPay,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
            amount: Amount::IssuedCurrencyAmount(crate::models::IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "NaN".into(),
            }),
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }
}
