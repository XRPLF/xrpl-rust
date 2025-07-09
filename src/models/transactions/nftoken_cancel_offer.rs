use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::exceptions::XRPLNFTokenCancelOfferException;
use crate::models::{FlagCollection, NoFlags, ValidateCurrencies, XRPLModelResult};
use crate::models::{
    Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use super::{CommonFields, CommonTransactionBuilder};

/// Cancels existing token offers created using NFTokenCreateOffer.
///
/// See NFTokenCancelOffer:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokencanceloffer>`
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
pub struct NFTokenCancelOffer<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// An array of IDs of the NFTokenOffer objects to cancel (not the IDs of NFToken
    /// objects, but the IDs of the NFTokenOffer objects). Each entry must be a
    /// different object ID of an NFTokenOffer object; the transaction is invalid
    /// if the array contains duplicate entries.
    #[serde(borrow)]
    #[serde(rename = "NFTokenOffers")]
    pub nftoken_offers: Vec<Cow<'a, str>>,
}

impl<'a: 'static> Model for NFTokenCancelOffer<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_nftoken_offers_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for NFTokenCancelOffer<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for NFTokenCancelOffer<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> NFTokenCancelOfferError for NFTokenCancelOffer<'a> {
    fn _get_nftoken_offers_error(&self) -> XRPLModelResult<()> {
        if self.nftoken_offers.is_empty() {
            Err(XRPLNFTokenCancelOfferException::CollectionEmpty {
                field: "nftoken_offers".into(),
                r#type: stringify!(Vec).into(),
            }
            .into())
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenCancelOffer<'a> {
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
        nftoken_offers: Vec<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::NFTokenCancelOffer,
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
            nftoken_offers,
        }
    }

    /// Add offer to cancel
    pub fn add_offer(mut self, offer_id: Cow<'a, str>) -> Self {
        self.nftoken_offers.push(offer_id);
        self
    }

    /// Set offers to cancel
    pub fn with_offers(mut self, offers: Vec<Cow<'a, str>>) -> Self {
        self.nftoken_offers = offers;
        self
    }
}

pub trait NFTokenCancelOfferError {
    fn _get_nftoken_offers_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::models::Model;

    #[test]
    fn test_nftoken_offer_error() {
        let nftoken_cancel_offer = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            nftoken_offers: Vec::new(), // Empty vec should cause error
        };

        assert_eq!(
            nftoken_cancel_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The value of the field `\"nftoken_offers\"` is not allowed to be empty (type `\"Vec\"`). If the field is optional, define it to be `None`"
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_offers: vec![
                "9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D".into(),
            ],
        };

        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"NFTokenCancelOffer","Flags":0,"SigningPubKey":"","NFTokenOffers":["9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"]}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenCancelOffer = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let nftoken_cancel_offer = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            nftoken_offers: vec![
                "9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D".into(),
            ],
        }
        .add_offer("1A2B3C4D5E6F7890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into())
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("canceling NFT offers".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(nftoken_cancel_offer.nftoken_offers.len(), 2);
        assert_eq!(
            nftoken_cancel_offer.nftoken_offers[0],
            "9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"
        );
        assert_eq!(
            nftoken_cancel_offer.nftoken_offers[1],
            "1A2B3C4D5E6F7890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890"
        );
        assert_eq!(
            nftoken_cancel_offer.common_fields.fee.as_ref().unwrap().0,
            "12"
        );
        assert_eq!(nftoken_cancel_offer.common_fields.sequence, Some(123));
        assert_eq!(
            nftoken_cancel_offer.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(nftoken_cancel_offer.common_fields.source_tag, Some(12345));
        assert_eq!(
            nftoken_cancel_offer
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_default() {
        let nftoken_cancel_offer = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            nftoken_offers: vec![
                "9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D".into(),
            ],
        };

        assert_eq!(
            nftoken_cancel_offer.common_fields.account,
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"
        );
        assert_eq!(
            nftoken_cancel_offer.common_fields.transaction_type,
            TransactionType::NFTokenCancelOffer
        );
        assert_eq!(nftoken_cancel_offer.nftoken_offers.len(), 1);
        assert!(nftoken_cancel_offer.common_fields.fee.is_none());
        assert!(nftoken_cancel_offer.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_offers() {
        let cancel_multiple = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "rCancelAccount123".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_offers(vec![
            "OFFER1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
            "OFFER2468ACEF13579BDF024681357ACE9BDF13579CE024681357BDF024681".into(),
            "OFFER369CFBEA147D258E047AD158FB269D147D258FB047AD158E047AD158E0".into(),
        ])
        .with_fee("15".into())
        .with_sequence(456);

        assert_eq!(cancel_multiple.nftoken_offers.len(), 3);
        assert_eq!(cancel_multiple.common_fields.fee.as_ref().unwrap().0, "15");
        assert_eq!(cancel_multiple.common_fields.sequence, Some(456));
        assert!(cancel_multiple.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_cancel = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "rTicketUser111".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            nftoken_offers: vec![
                "9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D".into(),
            ],
        }
        .with_ticket_sequence(789)
        .with_fee("12".into());

        assert_eq!(ticket_cancel.common_fields.ticket_sequence, Some(789));
        assert_eq!(ticket_cancel.nftoken_offers.len(), 1);
        // When using tickets, sequence should be None or 0
        assert!(ticket_cancel.common_fields.sequence.is_none());
    }

    #[test]
    fn test_add_offer_incrementally() {
        let incremental_cancel = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "rIncrementalUser222".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            ..Default::default()
        }
        .add_offer("FIRST1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into())
        .add_offer("SECOND2468ACEF13579BDF024681357ACE9BDF13579CE024681357BDF024681".into())
        .add_offer("THIRD369CFBEA147D258E047AD158FB269D147D258FB047AD158E047AD158E0".into())
        .with_fee("18".into())
        .with_sequence(789);

        assert_eq!(incremental_cancel.nftoken_offers.len(), 3);
        assert_eq!(
            incremental_cancel.nftoken_offers[0],
            "FIRST1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890"
        );
        assert_eq!(
            incremental_cancel.nftoken_offers[1],
            "SECOND2468ACEF13579BDF024681357ACE9BDF13579CE024681357BDF024681"
        );
        assert_eq!(
            incremental_cancel.nftoken_offers[2],
            "THIRD369CFBEA147D258E047AD158FB269D147D258FB047AD158E047AD158E0"
        );
        assert_eq!(incremental_cancel.common_fields.sequence, Some(789));
        assert!(incremental_cancel.validate().is_ok());
    }

    #[test]
    fn test_with_memo_and_source_tag() {
        let memo_cancel = NFTokenCancelOffer {
            common_fields: CommonFields {
                account: "rMemoUser333".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                ..Default::default()
            },
            nftoken_offers: vec![
                "MEMO1234567890ABCDEF1234567890FEDCBA0987654321ABCDEF1234567890".into(),
            ],
        }
        .with_memo(Memo {
            memo_data: Some("bulk cancel".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("cleanup".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_source_tag(98765)
        .with_fee("12".into())
        .with_sequence(111);

        assert_eq!(memo_cancel.common_fields.memos.as_ref().unwrap().len(), 2);
        assert_eq!(memo_cancel.common_fields.source_tag, Some(98765));
        assert_eq!(memo_cancel.common_fields.sequence, Some(111));
        assert!(memo_cancel.validate().is_ok());
    }
}
