use crate::Err;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::XRPLNFTokenCancelOfferException;
use crate::models::{
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

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
    #[serde(default = "TransactionType::nftoken_cancel_offer")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: &'a str,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<&'a str>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<&'a str>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<&'a str>,
    /// Set of bit-flags for this transaction.
    pub flags: Option<u32>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the NFTokenCancelOffer model.
    ///
    /// See NFTokenCancelOffer fields:
    /// `<https://xrpl.org/nftokencanceloffer.html#nftokencanceloffer-fields>`
    /// Lifetime issue
    #[serde(borrow)]
    #[serde(rename = "NFTokenOffers")]
    pub nftoken_offers: Vec<&'a str>,
}

impl<'a> Default for NFTokenCancelOffer<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::NFTokenCancelOffer,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            nftoken_offers: Default::default(),
        }
    }
}

impl<'a: 'static> Model for NFTokenCancelOffer<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_nftoken_offers_error() {
            Ok(_) => Ok(()),
            Err(error) => Err!(error),
        }
    }
}

impl<'a> Transaction for NFTokenCancelOffer<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> NFTokenCancelOfferError for NFTokenCancelOffer<'a> {
    fn _get_nftoken_offers_error(&self) -> Result<(), XRPLNFTokenCancelOfferException> {
        if self.nftoken_offers.is_empty() {
            Err(XRPLNFTokenCancelOfferException::CollectionEmpty {
                field: "nftoken_offers",
                r#type: stringify!(Vec),
                resource: "",
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenCancelOffer<'a> {
    pub fn new(
        account: &'a str,
        nftoken_offers: Vec<&'a str>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::NFTokenCancelOffer,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags: None,
            memos,
            signers,
            nftoken_offers,
        }
    }
}

pub trait NFTokenCancelOfferError {
    fn _get_nftoken_offers_error(&self) -> Result<(), XRPLNFTokenCancelOfferException>;
}

#[cfg(test)]
mod test_nftoken_cancel_offer_error {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use crate::models::Model;

    use super::*;

    #[test]
    fn test_nftoken_offer_error() {
        let nftoken_cancel_offer = NFTokenCancelOffer {
            transaction_type: TransactionType::NFTokenCancelOffer,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            nftoken_offers: Vec::new(),
        };

        assert_eq!(
            nftoken_cancel_offer.validate().unwrap_err().to_string().as_str(),
            "The value of the field `nftoken_offers` is not allowed to be empty (type `Vec`). If the field is optional, define it to be `None`. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = NFTokenCancelOffer::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
            vec!["9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"NFTokenCancelOffer","Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","NFTokenOffers":["9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"]}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = NFTokenCancelOffer::new(
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX",
            vec!["9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"NFTokenCancelOffer","Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","NFTokenOffers":["9C92E061381C1EF37A8CDE0E8FC35188BFC30B1883825042A64309AC09F4C36D"]}"#;

        let txn_as_obj: NFTokenCancelOffer = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
