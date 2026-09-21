use alloc::{borrow::Cow, string::ToString, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
};

use super::{CommonFields, Transaction, TransactionType};

/// Deposits first-loss capital into a LoanBroker ledger
/// entry to provide protection for vault depositors.
/// Only the owner of the associated LoanBroker entry
/// can initiate this transaction.
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
pub struct LoanBrokerCoverDeposit<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The Loan Broker ID that the transaction is modifying.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Cow<'a, str>,
    /// The First-Loss Capital amount to deposit.
    pub amount: Amount<'a>,
}

impl Model for LoanBrokerCoverDeposit<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()?;

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

        validate_hash256("loan_broker_id", &self.loan_broker_id)?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerCoverDeposit<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerCoverDeposit<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerCoverDeposit<'a> {
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
        loan_broker_id: Cow<'a, str>,
        amount: Amount<'a>,
    ) -> LoanBrokerCoverDeposit<'a> {
        LoanBrokerCoverDeposit {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerCoverDeposit,
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

    /// Set the Amount field.
    pub fn with_amount(mut self, amount: Amount<'a>) -> Self {
        self.amount = amount;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::models::IssuedCurrencyAmount;
    use alloc::format;

    use super::*;

    const SOURCE: &str = "rEXAMPLE9AbCdEfGhIjKlMnOpQrStUvWxYz";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    fn base_tx(
        loan_broker_id: &'static str,
        amount: Amount<'static>,
    ) -> LoanBrokerCoverDeposit<'static> {
        LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: loan_broker_id.into(),
            amount,
        }
    }

    fn xrp(value: &'static str) -> Amount<'static> {
        Amount::XRPAmount(XRPAmount::from(value))
    }

    fn usd(value: &'static str) -> Amount<'static> {
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
            currency: "USD".into(),
            issuer: "rIssuer1234567890abcdef1234567890abcdef".into(),
            value: value.into(),
        })
    }

    #[test]
    fn test_serde() {
        let tx = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                fee: Some(XRPAmount::from("12")),
                account: SOURCE.into(),
                sequence: Some(8),
                last_ledger_sequence: Some(7108682),
                transaction_type: TransactionType::LoanBrokerCoverDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rIssuer1234567890abcdef1234567890abcdef".into(),
                value: "1000".into(),
            }),
        };

        let default_json_str = r#"{"TransactionType":"LoanBrokerCoverDeposit","Account":"rEXAMPLE9AbCdEfGhIjKlMnOpQrStUvWxYz","Fee":"12","Flags":0,"SigningPubKey":"","LastLedgerSequence":7108682,"Sequence":8,"LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":{"currency":"USD","issuer":"rIssuer1234567890abcdef1234567890abcdef","value":"1000"}}"#;

        let default_json_value: serde_json::Value =
            serde_json::from_str(default_json_str).expect("Failed to deserialize");
        let serialized_tx = serde_json::to_value(&tx).expect("Failed to serialize to value");

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerCoverDeposit =
            serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_amount() {
        let tx = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF123456789".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
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
        let mut tx = LoanBrokerCoverDeposit::new(
            SOURCE.into(),
            None,
            Some(XRPAmount::from("12")),
            Some(7108682),
            Some(alloc::vec![Memo {
                memo_data: Some("6465706F736974".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(8),
            None,
            Some(12345),
            None,
            LOAN_BROKER_ID.into(),
            Amount::XRPAmount(XRPAmount::from("1000000")),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverDeposit
        );
        assert_eq!(tx.get_common_fields().account, SOURCE);
        assert_eq!(tx.get_common_fields().sequence, Some(8));
        assert_eq!(tx.loan_broker_id, LOAN_BROKER_ID);
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverDeposit,
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            ..Default::default()
        }
        .with_amount(Amount::XRPAmount(XRPAmount::from("1000000")))
        .with_fee("12".into())
        .with_sequence(8)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("6465706F736974".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.amount, Amount::XRPAmount(XRPAmount::from("1000000")));
        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(8));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
        assert!(tx.get_errors().is_ok());
    }

    /// The amount value is read from whichever `Amount` variant is supplied, so
    /// each variant has to be accepted by `get_errors`.
    #[test]
    fn test_valid_issued_currency_and_mpt_amounts() {
        let base = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
        };

        let issued = LoanBrokerCoverDeposit {
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: "rH5gvkKxGHrFAMAACeu9CB3FMu7pQY7Zh4".into(),
                value: "1000".into(),
            }),
            ..base.clone()
        };
        assert!(issued.get_errors().is_ok());

        let mpt = LoanBrokerCoverDeposit {
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
        let tx = LoanBrokerCoverDeposit {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerCoverDeposit,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
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

    #[test]
    fn test_invalid_loan_broker_id_empty() {
        let tx = base_tx("", xrp("1000000"));
        assert!(tx.get_errors().is_err());

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id_too_long() {
        // 66 hex chars instead of 64
        let tx = LoanBrokerCoverDeposit {
            loan_broker_id: format!("{}AB", LOAN_BROKER_ID).into(),
            ..base_tx(LOAN_BROKER_ID, xrp("1000000"))
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id_non_hex() {
        // Correct length (64) but starts with a non-hex character
        let tx = base_tx(
            "Z123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD",
            xrp("1000000"),
        );

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    #[test]
    fn test_invalid_zero_amount() {
        let tx = base_tx(LOAN_BROKER_ID, xrp("0"));

        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_xrp_amount() {
        assert!(base_tx(LOAN_BROKER_ID, xrp("1000000")).get_errors().is_ok());
    }

    #[test]
    fn test_valid_issued_currency_amount() {
        assert!(base_tx(LOAN_BROKER_ID, usd("1000")).get_errors().is_ok());
    }

    #[test]
    fn test_new_and_with_amount() {
        let tx = LoanBrokerCoverDeposit::new(
            SOURCE.into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            LOAN_BROKER_ID.into(),
            xrp("0"),
        );

        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerCoverDeposit
        );
        // Starts invalid (zero amount), then is fixed via the builder method
        assert!(tx.get_errors().is_err());

        let tx = tx.with_amount(xrp("1000000"));

        assert_eq!(tx.amount, xrp("1000000"));
        assert!(tx.get_errors().is_ok());
    }
}
