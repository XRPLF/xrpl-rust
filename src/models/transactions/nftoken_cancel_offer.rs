use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::exceptions::XRPLNFTokenCancelOfferException;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags, XRPLModelResult};

use super::CommonFields;

/// Cancels existing token offers created using NFTokenCreateOffer.
///
/// See NFTokenCancelOffer:
/// `<https://xrpl.org/nftokencanceloffer.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenCancelOffer<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the NFTokenCancelOffer model.
    //
    // See NFTokenCancelOffer fields:
    // `<https://xrpl.org/nftokencanceloffer.html#nftokencanceloffer-fields>`
    // Lifetime issue
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

        Ok(())
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

impl<'a> Default for NFTokenCancelOffer<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::NFTokenCancelOffer,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_offers: Vec::new(),
        }
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

    /// Add offer to cancel6
    pub fn add_offer(mut self, offer_id: Cow<'a, str>) -> Self {
        self.nftoken_offers.push(offer_id);
        self
    }

    /// Set offers to cancel
    pub fn with_offers(mut self, offers: Vec<Cow<'a, str>>) -> Self {
        self.nftoken_offers = offers;
        self
    }

    /// Set fee
    pub fn with_fee(mut self, fee: XRPAmount<'a>) -> Self {
        self.common_fields.fee = Some(fee);
        self
    }

    /// Set sequence
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.common_fields.sequence = Some(sequence);
        self
    }

    /// Set last ledger sequence
    pub fn with_last_ledger_sequence(mut self, last_ledger_sequence: u32) -> Self {
        self.common_fields.last_ledger_sequence = Some(last_ledger_sequence);
        self
    }

    /// Add memo
    pub fn with_memo(mut self, memo: Memo) -> Self {
        if let Some(ref mut memos) = self.common_fields.memos {
            memos.push(memo);
        } else {
            self.common_fields.memos = Some(vec![memo]);
        }
        self
    }

    /// Set source tag
    pub fn with_source_tag(mut self, source_tag: u32) -> Self {
        self.common_fields.source_tag = Some(source_tag);
        self
    }

    /// Set ticket sequence
    pub fn with_ticket_sequence(mut self, ticket_sequence: u32) -> Self {
        self.common_fields.ticket_sequence = Some(ticket_sequence);
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
            nftoken_cancel_offer.validate().unwrap_err().to_string().as_str(),
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
}
