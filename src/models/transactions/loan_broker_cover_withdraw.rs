use alloc::{borrow::Cow, vec::Vec};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelException,
};

use super::{CommonFields, Transaction, TransactionType};

const LOAN_BROKER_ID_HEX_LEN: usize = 64;

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
pub struct LoanBrokerCoverWithdraw<'a> {
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
    /// An account to receive the assets. It must be able to receive the asset.
    pub destination: Option<Cow<'a, str>>,
    /// Arbitrary tag identifying the reason for the transaction to the destination.
    pub destination_tag: Option<u32>,
}

impl Model for LoanBrokerCoverWithdraw<'_> {
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

        Self::validate_loan_broker_id(&self.loan_broker_id)?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerCoverWithdraw<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerCoverWithdraw<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerCoverWithdraw<'a> {
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
        destination: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
    ) -> LoanBrokerCoverWithdraw<'a> {
        LoanBrokerCoverWithdraw {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerCoverWithdraw,
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
            destination,
            destination_tag,
        }
    }

    /// Set the Destination field.
    pub fn with_destination(mut self, destination: Cow<'a, str>) -> Self {
        self.destination = Some(destination);
        self
    }

    /// Set the DestinationTag field.
    pub fn with_destination_tag(mut self, destination_tag: u32) -> Self {
        self.destination_tag = Some(destination_tag);
        self
    }

    fn validate_loan_broker_id(value: &str) -> Result<(), XRPLModelException> {
        if value.len() != LOAN_BROKER_ID_HEX_LEN {
            return Err(XRPLModelException::InvalidValueFormat {
                field: "loan_broker_id".to_string(),
                format: "64 hex characters (256-bit hash)".to_string(),
                found: value.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = "r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";
    const DESTINATION: &str = "rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR";

    #[test]
    fn test_serde() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9","TransactionType":"LoanBrokerCoverWithdraw","Flags":0,"SigningPubKey":"","LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD","Amount":"1000000","Destination":"rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR","DestinationTag":32}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerCoverWithdraw =
            serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_valid() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
        };

        assert!(tx.get_errors().is_ok())
    }

    #[test]
    fn test_invalid_amount() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF123456789".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }
}
