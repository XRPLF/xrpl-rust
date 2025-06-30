use alloc::borrow::Cow;
use alloc::vec::Vec;
use bigdecimal::{BigDecimal, Zero};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, XRPLModelException, XRPLModelResult,
};

use crate::models::amount::{Amount, XRPAmount};
use crate::models::transactions::exceptions::XRPLNFTokenCreateOfferException;

use super::{CommonFields, FlagCollection};

/// Transactions of the NFTokenCreateOffer type support additional values
/// in the Flags field. This enum represents those options.
///
/// See NFTokenCreateOffer flags:
/// `<https://xrpl.org/nftokencreateoffer.html#nftokencreateoffer-flags>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum NFTokenCreateOfferFlag {
    /// If enabled, indicates that the offer is a sell offer.
    /// Otherwise, it is a buy offer.
    TfSellOffer = 0x00000001,
}

/// Creates either a new Sell offer for an NFToken owned by
/// the account executing the transaction, or a new Buy
/// offer for an NFToken owned by another account.
///
/// See NFTokenCreateOffer:
/// `<https://xrpl.org/nftokencreateoffer.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenCreateOffer<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NFTokenCreateOfferFlag>,
    // The custom fields for the NFTokenCreateOffer model.
    //
    // See NFTokenCreateOffer fields:
    // `<https://xrpl.org/nftokencreateoffer.html#nftokencreateoffer-fields>`
    /// Identifies the NFToken object that the offer references.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    /// Indicates the amount expected or offered for the corresponding NFToken.
    /// The amount must be non-zero, except where this is an offer to sell and the
    /// asset is XRP; then, it is legal to specify an amount of zero, which means
    /// that the current owner of the token is giving it away, gratis, either to
    /// anyone at all, or to the account identified by the Destination field.
    pub amount: Amount<'a>,
    /// Who owns the corresponding NFToken. If the offer is to buy a token, this field
    /// must be present and it must be different than the Account field (since an offer
    /// to buy a token one already holds is meaningless). If the offer is to sell a token,
    /// this field must not be present, as the owner is, implicitly, the same as the
    /// Account (since an offer to sell a token one doesn't already hold is meaningless)
    pub owner: Option<Cow<'a, str>>,
    /// Time after which the offer is no longer active, in seconds since the Ripple Epoch.
    pub expiration: Option<u32>,
    /// If present, indicates that this offer may only be accepted by the specified account.
    /// Attempts by other accounts to accept this offer MUST fail.
    pub destination: Option<Cow<'a, str>>,
}

impl<'a: 'static> Model for NFTokenCreateOffer<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_amount_error()?;
        self._get_destination_error()?;
        self._get_owner_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, NFTokenCreateOfferFlag> for NFTokenCreateOffer<'a> {
    fn has_flag(&self, flag: &NFTokenCreateOfferFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NFTokenCreateOfferFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NFTokenCreateOfferFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> Default for NFTokenCreateOffer<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_id: "".into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            owner: None,
            expiration: None,
            destination: None,
        }
    }
}

impl<'a> NFTokenCreateOfferError for NFTokenCreateOffer<'a> {
    fn _get_amount_error(&self) -> XRPLModelResult<()> {
        let amount_into_decimal: BigDecimal = self.amount.clone().try_into()?;
        if !self.has_flag(&NFTokenCreateOfferFlag::TfSellOffer) && amount_into_decimal.is_zero() {
            Err(XRPLModelException::ValueZero("amount".into()))
        } else {
            Ok(())
        }
    }

    fn _get_destination_error(&self) -> XRPLModelResult<()> {
        if let Some(destination) = &self.destination {
            if destination == &self.common_fields.account {
                Err(XRPLModelException::ValueEqualsValue {
                    field1: "destination".into(),
                    field2: "account".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_owner_error(&self) -> XRPLModelResult<()> {
        if let Some(owner) = &self.owner {
            if self.has_flag(&NFTokenCreateOfferFlag::TfSellOffer) {
                Err(XRPLNFTokenCreateOfferException::IllegalOption {
                    field: "owner".into(),
                    context: "NFToken sell offers".into(),
                }
                .into())
            } else if owner == &self.common_fields.account {
                Err(XRPLModelException::ValueEqualsValue {
                    field1: "owner".into(),
                    field2: "account".into(),
                })
            } else {
                Ok(())
            }
        } else if !self.has_flag(&NFTokenCreateOfferFlag::TfSellOffer) {
            Err(XRPLNFTokenCreateOfferException::OptionRequired {
                field: "owner".into(),
                context: "NFToken buy offers".into(),
            }
            .into())
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenCreateOffer<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<NFTokenCreateOfferFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        nftoken_id: Cow<'a, str>,
        destination: Option<Cow<'a, str>>,
        expiration: Option<u32>,
        owner: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::NFTokenCreateOffer,
                account_txn_id,
                fee,
                Some(flags.unwrap_or_default()),
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
            nftoken_id,
            amount,
            owner,
            expiration,
            destination,
        }
    }

    /// Set owner
    pub fn with_owner(mut self, owner: Cow<'a, str>) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Set destination
    pub fn with_destination(mut self, destination: Cow<'a, str>) -> Self {
        self.destination = Some(destination);
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

    /// Add flag
    pub fn with_flag(mut self, flag: NFTokenCreateOfferFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    /// Set multiple flags
    pub fn with_flags(mut self, flags: Vec<NFTokenCreateOfferFlag>) -> Self {
        self.common_fields.flags = flags.into();
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

pub trait NFTokenCreateOfferError {
    fn _get_amount_error(&self) -> XRPLModelResult<()>;
    fn _get_destination_error(&self) -> XRPLModelResult<()>;
    fn _get_owner_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::models::amount::{Amount, XRPAmount};
    use crate::models::Model;

    #[test]
    fn test_amount_error() {
        let nftoken_create_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "".into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            ..Default::default()
        };

        assert_eq!(
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The value of the field `\"amount\"` is not allowed to be zero"
        );
    }

    #[test]
    fn test_destination_error() {
        let nftoken_create_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            destination: Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"destination\"` is not allowed to be the same as the value of the field `\"account\"`"
        );
    }

    #[test]
    fn test_owner_error() {
        let mut nftoken_create_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                flags: vec![NFTokenCreateOfferFlag::TfSellOffer].into(),
                ..Default::default()
            },
            nftoken_id: "".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            owner: Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The optional field `\"owner\"` is not allowed to be defined for \"NFToken sell offers\""
        );

        nftoken_create_offer.common_fields.flags = FlagCollection::default();
        nftoken_create_offer.owner = None;

        assert_eq!(
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The optional field `\"owner\"` is required to be defined for \"NFToken buy offers\""
        );

        nftoken_create_offer.owner = Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into());

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"owner\"` is not allowed to be the same as the value of the field `\"account\"`"
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                flags: vec![NFTokenCreateOfferFlag::TfSellOffer].into(),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        };

        let default_json_str = r#"{"Account":"rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX","TransactionType":"NFTokenCreateOffer","Flags":1,"SigningPubKey":"","NFTokenID":"000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007","Amount":"1000000"}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: NFTokenCreateOffer = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
