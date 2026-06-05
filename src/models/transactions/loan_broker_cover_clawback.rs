use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, IssuedCurrencyAmount, Model, NoFlags, ValidateCurrencies, XRPAmount,
    XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

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
pub struct LoanBrokerCoverClawback<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The Loan Broker ID from which to clawback First-Loss Capital.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Option<Cow<'a, str>>,
    /// The First-Loss Capital amount to clawback.
    /// If the amount is 0 or not provided, clawback funds up to LoanBroker.DebtTotal * LoanBroker.CoverRateMinimum.
    pub amount: Option<Amount<'a>>,
}

impl Model for LoanBrokerCoverClawback<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;

        //Amount must not be XRP
        if let Some(Amount::XRPAmount(..)) = &self.amount {
            return Err(XRPLModelException::InvalidValue {
                field: "amount".into(),
                expected: "IssuedCurrencyAmount(IOU or MPT)".into(),
                found: "XRPAmount".into(),
            });
        }

        self.validate_field_requirements()
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerCoverClawback<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerCoverClawback<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerCoverClawback<'a> {
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
        loan_broker_id: Option<Cow<'a, str>>,
        amount: Option<Amount<'a>>,
    ) -> LoanBrokerCoverClawback<'a> {
        LoanBrokerCoverClawback {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerCoverClawback,
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
            loan_broker_id,
            amount,
        }
    }

    /// Set the LoanBrokerID field.
    pub fn with_loan_broker_id(mut self, loan_broker_id: Cow<'a, str>) -> Self {
        self.loan_broker_id = Some(loan_broker_id);
        self
    }

    /// Set the Amount field.
    pub fn with_amount(mut self, amount: Amount<'a>) -> Self {
        self.amount = Some(amount);
        self
    }

    fn validate_field_requirements(&self) -> XRPLModelResult<()> {
        match (&self.loan_broker_id, &self.amount) {
            // Amount present without loan_broker_id
            (None, Some(_)) => self.validate_amount_without_broker(),
            (Some(_), None) => Err(XRPLModelException::FieldRequiresField {
                field1: "loan_broker_id".into(),
                field2: "amount".into(),
            }),
            // Neither field is present
            (None, None) => Err(XRPLModelException::MissingField(
                "'loan_broker_id' and 'amount'".into(),
            )),
            // Both present
            (Some(_), Some(_)) => Ok(()),
        }
    }

    fn validate_amount_without_broker(&self) -> XRPLModelResult<()> {
        match &self.amount {
            Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount { issuer, .. })) => {
                // Issuer must not be the submitter
                let issuer_is_submitter = *issuer == self.common_fields.account;
                if issuer_is_submitter {
                    Err(XRPLModelException::InvalidValue {
                        field: "amount.issuer".into(),
                        expected: "Issuer account".into(),
                        found: "submitter account".into(),
                    })
                } else {
                    Ok(())
                }
            }
            // XRP already rejected
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "r9LqNeG6qHxLoanBrokerCoverClawback5weJ9mZgQ";
    const LOAN_BROKER_ID: &str = "rDB303FC1C7611B22C09E773B51044F6BEA02EF9";

    #[test]
    fn test_serde() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: Some(LOAN_BROKER_ID.into()),
            amount: Some(Amount::XRPAmount(XRPAmount::from("1000000"))),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverClawback5weJ9mZgQ","TransactionType":"LoanBrokerCoverClawback","Flags":0,"SigningPubKey":"","LoanBrokerID":"rDB303FC1C7611B22C09E773B51044F6BEA02EF9","Amount":"1000000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(&serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerCoverClawback =
            serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_no_amount_no_loan_broker_id_specified() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: None,
            amount: None,
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::MissingField(..))
        ))
    }

    #[test]
    fn test_invalid_xrp_amount() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: Some(LOAN_BROKER_ID.into()),
            amount: Some(Amount::XRPAmount(XRPAmount("1000".into()))),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ))
    }

    #[test]
    fn test_invalid_same_issuer_same_submitter() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: None,
            amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: SOURCE.into(),
                value: "1000".into(),
            })),
        };

        dbg!(&tx.get_errors().err());

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ))
    }

    #[test]
    fn test_valid_loan_broker_cover_clawback() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: Some(LOAN_BROKER_ID.into()),
            amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: LOAN_BROKER_ID.into(),
                value: "1000".into(),
            })),
        };

        assert!(tx.get_errors().is_ok());
    }
}
