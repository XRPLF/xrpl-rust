use alloc::{borrow::Cow, string::ToString, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, IssuedCurrencyAmount, Model, NoFlags, ValidateCurrencies, XRPAmount,
    XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

/// The LoanBrokerCoverClawback transaction claws back first-loss
/// capital from a LoanBroker ledger entry. The transaction can
/// only be submitted by the issuer of the asset used in the
/// lending protocol, and can't clawback an amount that
/// would cause the available first-loss capital to drop below
/// the minimum amount defined by the LoanBroker.CoverRateMinimum value.
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

        match &self.amount {
            Some(Amount::MPTAmount(amount)) => {
                Self::validate_non_zero_negative_amount(amount.value.as_ref())?;
            }
            Some(Amount::IssuedCurrencyAmount(amount)) => {
                Self::validate_non_zero_negative_amount(amount.value.as_ref())?;
            }
            Some(Amount::XRPAmount(_)) => {
                return Err(XRPLModelException::InvalidValue {
                    field: "amount".into(),
                    expected: "IssuedCurrencyAmount(IOU or MPT)".into(),
                    found: "XRPAmount".into(),
                });
            }
            None => {}
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
            (None, Some(amount)) => self.validate_amount_without_broker(amount),
            (Some(v), _) => {
                validate_hash256("loan_broker_id", v)?;

                Ok(())
            }
            // Neither field is present
            (None, None) => Err(XRPLModelException::MissingField(
                "'loan_broker_id' and(or) 'amount'".into(),
            )),
        }
    }

    fn validate_amount_without_broker(&self, amount: &Amount) -> XRPLModelResult<()> {
        match amount {
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount { issuer, .. }) => {
                // Issuer must not be the submitter

                if *issuer == self.common_fields.account {
                    Err(XRPLModelException::MissingField("loan_broker_id".into()))
                } else {
                    Ok(())
                }
            }
            Amount::MPTAmount(_) => Err(XRPLModelException::MissingField("loan_broker_id".into())),

            Amount::XRPAmount(_) => Ok(()),
        }
    }

    fn validate_non_zero_negative_amount(value: &str) -> Result<(), XRPLModelException> {
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
                expected: "a non-negative/non-zero amount".to_string(),
                found: value.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    const SOURCE: &str = "r9LqNeG6qHxLoanBrokerCoverClawback5weJ9mZgQ";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    fn base_tx(
        loan_broker_id: Option<&'static str>,
        amount: Option<Amount<'static>>,
    ) -> LoanBrokerCoverClawback<'static> {
        LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: loan_broker_id.map(Into::into),
            amount,
        }
    }

    /// IOU amount whose issuer differs from SOURCE (same issuer value as the existing valid test).
    fn iou(value: &'static str) -> Amount<'static> {
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
            currency: "USD".into(),
            issuer: LOAN_BROKER_ID.into(),
            value: value.into(),
        })
    }

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

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverClawback5weJ9mZgQ","TransactionType":"LoanBrokerCoverClawback","Flags":0,"SigningPubKey":"","LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":"1000000"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

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

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::MissingField(_))
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

    #[test]
    fn test_invalid_negative_amount() {
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
                value: "-1000".into(),
            })),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: Some(
                "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567".into(),
            ),
            amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: LOAN_BROKER_ID.into(),
                value: "1000".into(),
            })),
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
        let mut tx = LoanBrokerCoverClawback::new(
            SOURCE.into(),
            None,
            Some(XRPAmount::from("12")),
            Some(7108682),
            Some(alloc::vec![Memo {
                memo_data: Some("636C617762616B".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(8),
            None,
            Some(12345),
            None,
            Some(LOAN_BROKER_ID.into()),
            Some(Amount::MPTAmount(crate::models::MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: MPT_ISSUANCE_ID.into(),
            })),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverClawback
        );
        assert_eq!(tx.get_common_fields().account, SOURCE);
        assert_eq!(tx.get_common_fields().sequence, Some(8));
        assert_eq!(tx.loan_broker_id, Some(LOAN_BROKER_ID.into()));
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_loan_broker_id(LOAN_BROKER_ID.into())
        .with_amount(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
            currency: "USD".into(),
            issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
            value: "1000".into(),
        }))
        .with_fee("12".into())
        .with_sequence(8)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("636C617762616B".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.loan_broker_id, Some(LOAN_BROKER_ID.into()));
        assert!(tx.amount.is_some());
        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(8));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
        assert!(tx.get_errors().is_ok());
    }

    /// Without a `loan_broker_id` the issuer of the clawed-back IOU is the
    /// broker being drawn from, so an issuer other than the submitter is valid.
    #[test]
    fn test_valid_issued_currency_without_loan_broker_id() {
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
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "1000".into(),
            })),
        };

        assert!(tx.get_errors().is_ok());
    }

    /// An MPT amount cannot identify the broker on its own, so `loan_broker_id`
    /// is required alongside it.
    #[test]
    fn test_invalid_mpt_amount_without_loan_broker_id() {
        let tx = LoanBrokerCoverClawback {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: None,
            amount: Some(Amount::MPTAmount(crate::models::MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: MPT_ISSUANCE_ID.into(),
            })),
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::MissingField(..))
        ));
    }

    /// `IssuedCurrencyAmount` validates its value with `f64::parse`, which accepts
    /// `NaN` and the infinities. `BigDecimal` does not, so `validate_positive_amount`
    /// still has to report a format error for those values.
    #[test]
    fn test_unparsable_amount() {
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
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "NaN".into(),
            })),
        };

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id_empty() {
        let tx = base_tx(Some(""), Some(iou("1000")));

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_too_long() {
        // 66 hex chars instead of 64
        let tx = LoanBrokerCoverClawback {
            loan_broker_id: Some(format!("{}AB", LOAN_BROKER_ID).into()),
            ..base_tx(Some(LOAN_BROKER_ID), Some(iou("1000")))
        };

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_non_hex() {
        // Correct length (64) but starts with a non-hex character
        let tx = base_tx(
            Some("Z123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD"),
            Some(iou("1000")),
        );

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_without_amount() {
        // Broker ID is validated even when Amount is absent
        let tx = base_tx(Some("E123F4567890ABCDE123F4567"), None);

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_amount_zero() {
        let tx = base_tx(Some(LOAN_BROKER_ID), Some(iou("0")));

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_broker_id_only() {
        assert!(base_tx(Some(LOAN_BROKER_ID), None).get_errors().is_ok());
    }

    #[test]
    fn test_invalid_missing_broker_id_error_is_missing_field() {
        let tx = base_tx(None, None);

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::MissingField(_))
        ));
    }

    #[test]
    fn test_serde_roundtrip_iou_amount() {
        let tx = base_tx(Some(LOAN_BROKER_ID), Some(iou("1000")));

        let json = serde_json::to_string(&tx).unwrap();
        let roundtripped: LoanBrokerCoverClawback = serde_json::from_str(&json).unwrap();

        assert_eq!(tx, roundtripped);
    }

    #[test]
    fn test_new_and_builder_methods() {
        let tx = LoanBrokerCoverClawback::new(
            SOURCE.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverClawback
        );
        assert_eq!(tx.loan_broker_id, None);
        assert_eq!(tx.amount, None);
        // Starts invalid (neither field set), then is fixed via the builder methods
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::MissingField(_))
        ));

        let tx = tx.with_amount(iou("1000"));

        assert_eq!(tx.amount, Some(iou("1000")));
        assert!(tx.get_errors().is_ok());

        let tx = tx.with_loan_broker_id(LOAN_BROKER_ID.into());

        assert_eq!(tx.loan_broker_id, Some(LOAN_BROKER_ID.into()));
        assert!(tx.get_errors().is_ok());
    }
}
