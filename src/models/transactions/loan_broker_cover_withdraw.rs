use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::{CommonTransactionBuilder, Memo, Signer},
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount,
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
    /// The Fist-Loss Capital amount to deposit.
    pub amount: Amount<'a>,
    /// An account to receive the assets. It must be able to receive the asset.
    pub destination: Option<Cow<'a, str>>,
    /// Arbitrary tag identifying the reason for the transaction to the destination.
    pub destination_tag: Option<u32>,
}

impl Model for LoanBrokerCoverWithdraw<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACOUNT: &str = "r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9";
    const LOAN_BROKER_ID: &str = "DB303FC1C7611B22C09E773B51044F6BEA02EF9";
    const DESTINATION: &str = "rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR";

    #[test]
    fn test_serde() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACOUNT.into(),
                transaction_type: TransactionType::LoanBrokerCoverWithdraw,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            loan_broker_id: LOAN_BROKER_ID.into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: Some(DESTINATION.into()),
            destination_tag: Some(32),
        };

        let default_json_str = r#"{"Account":"r9LqNeG6qHxLoanBrokerCoverWithdraw5weJ9","TransactionType":"LoanBrokerCoverWithdraw","Flags":0,"SigningPubKey":"","LoanBrokerID":"rDB303FC1C7611B22C09E773B51044F6BEA02EF9","Amount":"1000000","Destination":"rf7HPydP4ihkFkSRHWFq34b4SXRc7GvPCR","DestinationTag":32}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_tx = serde_json::to_value(&serde_json::to_string(&tx).unwrap()).unwrap();

        assert_eq!(serialized_tx, default_json_value);

        let deserilized_tx: LoanBrokerCoverWithdraw =
            serde_json::from_str(default_json_str).unwrap();

        assert_eq!(tx, deserilized_tx);
    }

    #[test]
    fn test_valid() {
        let tx = LoanBrokerCoverWithdraw {
            common_fields: CommonFields {
                account: ACOUNT.into(),
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
}
