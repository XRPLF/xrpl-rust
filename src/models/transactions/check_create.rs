use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    amount::Amount,
    transactions::{Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonTransactionBuilder, Memo, Signer};

/// Create a Check object in the ledger, which is a deferred
/// payment that can be cashed by its intended destination.
///
/// See CheckCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/checkcreate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
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
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CheckCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
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
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        destination: Cow<'a, str>,
        send_max: Amount<'a>,
        destination_tag: Option<u32>,
        expiration: Option<u32>,
        invoice_id: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CheckCreate,
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
            destination,
            send_max,
            destination_tag,
            expiration,
            invoice_id,
        }
    }

    pub fn with_destination_tag(mut self, destination_tag: u32) -> Self {
        self.destination_tag = Some(destination_tag);
        self
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_invoice_id(mut self, invoice_id: Cow<'a, str>) -> Self {
        self.invoice_id = Some(invoice_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = CheckCreate {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCreate,
                fee: Some("12".into()),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            destination: "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
            send_max: "100000000".into(),
            destination_tag: Some(1),
            expiration: Some(570113521),
            invoice_id: Some(
                "6F1DFD1D0FE8A32E40E1F2C05CF1C15545BAB56B617F9C6C2D63A6B704BEF59B".into(),
            ),
        };

        let default_json_str = r#"{"Account":"rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo","TransactionType":"CheckCreate","Fee":"12","Flags":0,"SigningPubKey":"","Destination":"rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy","SendMax":"100000000","DestinationTag":1,"Expiration":570113521,"InvoiceID":"6F1DFD1D0FE8A32E40E1F2C05CF1C15545BAB56B617F9C6C2D63A6B704BEF59B"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CheckCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let check_create = CheckCreate {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCreate,
                ..Default::default()
            },
            destination: "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
            send_max: "100000000".into(),
            ..Default::default()
        }
        .with_destination_tag(1)
        .with_expiration(570113521)
        .with_invoice_id("6F1DFD1D0FE8A32E40E1F2C05CF1C15545BAB56B617F9C6C2D63A6B704BEF59B".into())
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            check_create.destination,
            "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy"
        );
        assert_eq!(check_create.destination_tag, Some(1));
        assert_eq!(check_create.expiration, Some(570113521));
        assert!(check_create.invoice_id.is_some());
        assert_eq!(check_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(check_create.common_fields.sequence, Some(123));
        assert_eq!(
            check_create.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(check_create.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_default() {
        let check_create = CheckCreate {
            common_fields: CommonFields {
                account: "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo".into(),
                transaction_type: TransactionType::CheckCreate,
                ..Default::default()
            },
            destination: "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
            send_max: "100000000".into(),
            ..Default::default()
        };

        assert_eq!(
            check_create.common_fields.account,
            "rUn84CUYbNjRoTQ6mSW7BVJPSVJNLb1QLo"
        );
        assert_eq!(
            check_create.common_fields.transaction_type,
            TransactionType::CheckCreate
        );
        assert_eq!(
            check_create.destination,
            "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy"
        );
        assert!(check_create.destination_tag.is_none());
        assert!(check_create.expiration.is_none());
        assert!(check_create.invoice_id.is_none());
    }
}
