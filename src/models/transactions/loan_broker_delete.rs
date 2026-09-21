use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{vault_common::validate_hash256, CommonTransactionBuilder, Memo, Signer},
    FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

/// Deletes a LoanBroker ledger entry. Only the owner
/// of the LoanBroker entry can delete it.
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
pub struct LoanBrokerDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The Loan Broker ID that the transaction is deleting.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Cow<'a, str>,
}

impl Model for LoanBrokerDelete<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;

        validate_hash256("loan_broker_id", &self.loan_broker_id)?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanBrokerDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanBrokerDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanBrokerDelete<'a> {
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
    ) -> LoanBrokerDelete<'a> {
        LoanBrokerDelete {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerDelete,
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
        }
    }

    /// Set the LoanBroker ID field.
    pub fn with_loan_broker_id(mut self, loan_broker_id: Cow<'a, str>) -> Self {
        self.loan_broker_id = loan_broker_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::models::XRPLModelException;
    use alloc::format;

    use super::*;

    const SOURCE: &str = "r9LqNeG6qHxLoanBrokerDeletter5weJ9mZgQ";
    const LOAN_BROKER_ID: &str = "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    fn base_tx(loan_broker_id: &'static str) -> LoanBrokerDelete<'static> {
        LoanBrokerDelete {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: loan_broker_id.into(),
        }
    }

    #[test]
    fn test_serde() {
        let tx = LoanBrokerDelete {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerDeletter5weJ9mZgQ","TransactionType":"LoanBrokerDelete","Flags":0,"SigningPubKey":"","LoanBrokerID":"E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerDelete = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_invalid_loan_broker_id_empty() {
        assert!(base_tx("").get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id() {
        let tx = LoanBrokerDelete {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF123".into(),
        };

        assert!(tx.get_errors().is_err());
        assert!(matches!(
            tx.get_errors().err(),
            Some(XRPLModelException::InvalidValueFormat { .. })
        ));
    }

    const VALID_LOAN_BROKER_ID: &str =
        "E123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD";

    #[test]
    fn test_new_and_accessors() {
        let mut tx = LoanBrokerDelete::new(
            SOURCE.into(),
            None,
            Some(XRPAmount::from("10")),
            Some(7108682),
            Some(alloc::vec![Memo {
                memo_data: Some("64656C6574696E67".into()),
                memo_format: None,
                memo_type: Some("74657874".into()),
            }]),
            Some(100),
            None,
            Some(12345),
            None,
            VALID_LOAN_BROKER_ID.into(),
        );

        assert!(tx.get_errors().is_ok());
        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerDelete
        );
        assert_eq!(tx.get_common_fields().account, SOURCE);
        assert_eq!(tx.get_common_fields().sequence, Some(100));
        assert_eq!(tx.loan_broker_id, VALID_LOAN_BROKER_ID);
        assert_eq!(
            Transaction::get_mut_common_fields(&mut tx).source_tag,
            Some(12345)
        );
    }

    #[test]
    fn test_builder_pattern() {
        let tx = LoanBrokerDelete {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanBrokerDelete,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_loan_broker_id(VALID_LOAN_BROKER_ID.into())
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_ticket_sequence(7)
        .with_memo(Memo {
            memo_data: Some("64656C6574696E67".into()),
            memo_format: None,
            memo_type: Some("74657874".into()),
        });

        assert_eq!(tx.loan_broker_id, VALID_LOAN_BROKER_ID);
        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(100));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(tx.common_fields.source_tag, Some(12345));
        assert_eq!(tx.common_fields.ticket_sequence, Some(7));
        assert_eq!(tx.common_fields.memos.as_ref().unwrap().len(), 1);
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_invalid_loan_broker_id_too_long() {
        // 66 hex chars instead of 64
        let tx = LoanBrokerDelete {
            loan_broker_id: format!("{}AB", LOAN_BROKER_ID).into(),
            ..base_tx(LOAN_BROKER_ID)
        };

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_non_hex() {
        // Correct length (64) but starts with a non-hex character
        let tx = base_tx("Z123F4567890ABCDE123F4567890ABCDEF1234567890ABCDEF1234567890ABCD");

        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_invalid_loan_broker_id_account_style_hash() {
        // 40-char value (the one used in test_serde) is not a valid 64-char hash256
        assert!(base_tx("rDB303FC1C7611B22C09E773B51044F6BEA02EF9")
            .get_errors()
            .is_err());
    }

    #[test]
    fn test_new_sets_transaction_type_and_id() {
        let tx = LoanBrokerDelete::new(
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
        );

        assert_eq!(
            tx.get_transaction_type(),
            &TransactionType::LoanBrokerDelete
        );
        assert_eq!(tx.loan_broker_id, LOAN_BROKER_ID);
        assert!(tx.get_errors().is_ok());
    }
}
