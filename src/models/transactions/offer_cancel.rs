use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, NoFlags};
use crate::models::{
    Model, ValidateCurrencies,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Removes an Offer object from the XRP Ledger.
///
/// See OfferCancel:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/offercancel>`
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
pub struct OfferCancel<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The sequence number (or Ticket number) of a previous OfferCreate transaction.
    /// If specified, cancel any offer object in the ledger that was created by that
    /// transaction. It is not considered an error if the offer specified does not exist.
    pub offer_sequence: u32,
}

impl<'a> Model for OfferCancel<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for OfferCancel<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for OfferCancel<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> OfferCancel<'a> {
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
        offer_sequence: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::OfferCancel,
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
            offer_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                fee: Some("12".into()),
                last_ledger_sequence: Some(7108629),
                sequence: Some(7),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            offer_sequence: 6,
        };

        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"OfferCancel","Fee":"12","Flags":0,"LastLedgerSequence":7108629,"Sequence":7,"SigningPubKey":"","OfferSequence":6}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: OfferCancel = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let offer_cancel = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                ..Default::default()
            },
            offer_sequence: 6,
        }
        .with_fee("12".into())
        .with_sequence(7)
        .with_last_ledger_sequence(7108629)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("canceling offer".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(offer_cancel.offer_sequence, 6);
        assert_eq!(offer_cancel.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(offer_cancel.common_fields.sequence, Some(7));
        assert_eq!(
            offer_cancel.common_fields.last_ledger_sequence,
            Some(7108629)
        );
        assert_eq!(offer_cancel.common_fields.source_tag, Some(12345));
        assert_eq!(offer_cancel.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let offer_cancel = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                ..Default::default()
            },
            offer_sequence: 6,
        };

        assert_eq!(
            offer_cancel.common_fields.account,
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"
        );
        assert_eq!(
            offer_cancel.common_fields.transaction_type,
            TransactionType::OfferCancel
        );
        assert_eq!(offer_cancel.offer_sequence, 6);
        assert!(offer_cancel.common_fields.fee.is_none());
        assert!(offer_cancel.common_fields.sequence.is_none());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_cancel = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                ..Default::default()
            },
            offer_sequence: 123,
        }
        .with_ticket_sequence(456)
        .with_fee("12".into());

        assert_eq!(ticket_cancel.common_fields.ticket_sequence, Some(456));
        assert_eq!(ticket_cancel.offer_sequence, 123);
        assert_eq!(ticket_cancel.common_fields.fee.as_ref().unwrap().0, "12");
        // When using tickets, sequence should be None or 0
        assert!(ticket_cancel.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_cancel = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                ..Default::default()
            },
            offer_sequence: 789,
        }
        .with_memo(Memo {
            memo_data: Some("first memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("second memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("12".into())
        .with_sequence(8);

        assert_eq!(multi_memo_cancel.offer_sequence, 789);
        assert_eq!(
            multi_memo_cancel
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_cancel.common_fields.sequence, Some(8));
    }

    #[test]
    fn test_minimal_cancel() {
        // Test canceling an offer with minimal fields
        let minimal_cancel = OfferCancel {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::OfferCancel,
                ..Default::default()
            },
            offer_sequence: 42,
        }
        .with_fee("10".into())
        .with_sequence(43);

        assert_eq!(minimal_cancel.offer_sequence, 42);
        assert_eq!(minimal_cancel.common_fields.sequence, Some(43));
        assert_eq!(minimal_cancel.common_fields.fee.as_ref().unwrap().0, "10");
        assert!(minimal_cancel.common_fields.memos.is_none());
        assert!(minimal_cancel.common_fields.source_tag.is_none());
    }
}
