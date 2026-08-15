use alloc::{borrow::Cow, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
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

    use super::*;

    const SOURCE: &str = "rEXAMPLE9AbCdEfGhIjKlMnOpQrStUvWxYz";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

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
}
