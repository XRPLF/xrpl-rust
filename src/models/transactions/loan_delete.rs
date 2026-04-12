use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
    FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount, XRPLModelResult,
};

use super::{CommonFields, Transaction, TransactionType};

/// Creates a new Loan ledger entry, representing a loan agreement
/// between a Loan Broker and Borrower.
/// The LoanSet transaction is a mutual agreement between
/// the Loan Broker and Borrower, and must be signed by both parties.
/// The following multi-signature flow can be initiated by either party:
/// 1. The borrower or loan broker creates the transaction with the
///     preagreed terms of the loan. They sign the transaction and
///     set the SigningPubKey, TxnSignature, Signers, Account,
///     Fee, Sequence, and Counterparty fields.
/// 2. The counterparty verifies the loan terms and signature
///     before signing and submitting the transaction.
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
pub struct LoanDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    #[serde(rename = "LoanID")]
    /// The ID of the Loan object to be deleted.
    pub loan_id: Cow<'a, str>,
}

impl Model for LoanDelete<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for LoanDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for LoanDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> LoanDelete<'a> {
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
        loan_id: Cow<'a, str>,
    ) -> LoanDelete<'a> {
        LoanDelete {
            common_fields: CommonFields::new(
                account,
                TransactionType::LoanBrokerSet,
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
            loan_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "r9LqNeG6qHxLoanDeleter6T5weJ9mZg";
    const LOAN_ID: &str = "rDB303FC1C7611B22C09E773B51044F6BE";

    #[test]
    fn test_invalid_data_too_long() {
        let tx = LoanDelete {
            common_fields: CommonFields {
                account: SOURCE.into(),
                transaction_type: TransactionType::LoanDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_id: LOAN_ID.into(),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanDeleter6T5weJ9mZg","TransactionType":"LoanDelete","Flags":0,"SigningPubKey":"","LoanID":"rDB303FC1C7611B22C09E773B51044F6BE"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(&serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanDelete = serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }
}
