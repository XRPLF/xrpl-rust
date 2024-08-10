use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::NoFlags;
use crate::models::{
    amount::Amount,
    model::Model,
    transactions::{Transaction, TransactionType},
};

use super::{Memo, Signer};

/// Create a Check object in the ledger, which is a deferred
/// payment that can be cashed by its intended destination.
///
/// See CheckCreate:
/// `<https://xrpl.org/checkcreate.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the CheckCreate model.
    //
    // See CheckCreate fields:
    // `<https://xrpl.org/checkcreate.html#checkcreate-fields>`
    /// The unique address of the account that can cash the Check.
    pub destination: Cow<'a, str>,
    /// Maximum amount of source currency the Check is allowed to debit the sender,
    /// including transfer fees on non-XRP currencies. The Check can only credit
    /// the destination with the same currency (from the same issuer, for non-XRP
    /// currencies). For non-XRP amounts, the nested field names MUST be lower-case.
    pub send_max: Amount<'a>,
    /// Arbitrary tag that identifies the reason for the Check, or a hosted recipient to pay.
    pub destination_tag: Option<u32>,
    /// Time after which the Check is no longer valid, in seconds since the Ripple Epoch.
    pub expiration: Option<u32>,
    /// Arbitrary 256-bit hash representing a specific reason or identifier for this Check.
    #[serde(rename = "InvoiceID")]
    pub invoice_id: Option<Cow<'a, str>>,
}

impl<'a> Model for CheckCreate<'a> {}

impl<'a> Transaction<'a, NoFlags> for CheckCreate<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CheckCreate<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        destination: Cow<'a, str>,
        send_max: Amount<'a>,
        destination_tag: Option<u32>,
        expiration: Option<u32>,
        invoice_id: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::CheckCreate,
                account_txn_id,
                fee,
                flags: None,
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            destination,
            send_max,
            destination_tag,
            expiration,
            invoice_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = CheckCreate::new(
            "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
            None,
            Some("12".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
            "100000000".into(),
            Some(1),
            Some(570113521),
            Some("6F1DFD1D0FE8A32E40E1F2C05CF1C15545BAB56B617F9C6C2D63A6B704BEF59B".into()),
        );
        let default_json_str = r#"{"Account":"rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo","TransactionType":"CheckCreate","Fee":"12","Destination":"rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy","SendMax":"100000000","DestinationTag":1,"Expiration":570113521,"InvoiceID":"6F1DFD1D0FE8A32E40E1F2C05CF1C15545BAB56B617F9C6C2D63A6B704BEF59B"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: CheckCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
