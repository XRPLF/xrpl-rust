use alloc::borrow::Cow;
use alloc::vec::Vec;
use bigdecimal::{BigDecimal, Zero};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use crate::models::amount::{Amount, XRPAmount};
use crate::models::transactions::exceptions::XRPLNFTokenCreateOfferException;

use super::{CommonFields, CommonTransactionBuilder, FlagCollection};

/// Transactions of the NFTokenCreateOffer type support additional values
/// in the Flags field. This enum represents those options.
///
/// See NFTokenCreateOffer flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokencreateoffer>`
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
/// `<https://xrpl.org/docs/references/protocol/transactions/types/nftokencreateoffer>`
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
pub struct NFTokenCreateOffer<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NFTokenCreateOfferFlag>,
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
        self.validate_currencies()
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

impl<'a> CommonTransactionBuilder<'a, NFTokenCreateOfferFlag> for NFTokenCreateOffer<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NFTokenCreateOfferFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
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
    use crate::models::Model;
    use crate::models::amount::{Amount, IssuedCurrencyAmount, XRPAmount};

    #[test]
    fn test_amount_error() {
        let nftoken_create_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
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
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            destination: Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
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
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            owner: Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into()),
            ..Default::default()
        };

        assert_eq!(
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
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
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
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

    #[test]
    fn test_builder_pattern() {
        let buy_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rBuyerAccount123".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        }
        .with_owner("rSellerAccount456".into()) // Required for buy offers
        .with_expiration(1672531200)
        .with_destination("rBuyerAccount123".into()) // Private offer
        .with_fee("12".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("buying NFT".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(
            buy_offer.nftoken_id,
            "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007"
        );
        assert_eq!(buy_offer.owner.as_ref().unwrap(), "rSellerAccount456");
        assert_eq!(buy_offer.expiration, Some(1672531200));
        assert_eq!(buy_offer.destination.as_ref().unwrap(), "rBuyerAccount123");
        assert!(!buy_offer.has_flag(&NFTokenCreateOfferFlag::TfSellOffer)); // Buy offer
        assert_eq!(buy_offer.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(buy_offer.common_fields.sequence, Some(123));
        assert_eq!(buy_offer.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(buy_offer.common_fields.source_tag, Some(12345));
        assert_eq!(buy_offer.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_sell_offer() {
        let sell_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rSellerAccount456".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "100".into(),
            )),
            ..Default::default()
        }
        .with_flag(NFTokenCreateOfferFlag::TfSellOffer)
        .with_expiration(1672531200)
        .with_fee("12".into())
        .with_sequence(456);

        assert!(sell_offer.has_flag(&NFTokenCreateOfferFlag::TfSellOffer));
        assert!(sell_offer.owner.is_none()); // Owner not allowed for sell offers
        assert_eq!(sell_offer.expiration, Some(1672531200));
        assert!(!sell_offer.amount.is_xrp()); // Selling for USD
        assert!(sell_offer.validate().is_ok());
    }

    #[test]
    fn test_free_giveaway() {
        let free_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rGiverAccount789".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("0")), // Free!
            ..Default::default()
        }
        .with_flag(NFTokenCreateOfferFlag::TfSellOffer) // Sell for 0 XRP
        .with_destination("rRecipientAccount999".into()) // Only this account can accept
        .with_fee("12".into())
        .with_sequence(789);

        assert!(free_offer.has_flag(&NFTokenCreateOfferFlag::TfSellOffer));
        assert_eq!(
            free_offer.destination.as_ref().unwrap(),
            "rRecipientAccount999"
        );
        assert!(free_offer.amount.is_xrp());
        assert!(free_offer.validate().is_ok()); // Zero amount allowed for sell offers
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rTicketUser111".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("500000")),
            ..Default::default()
        }
        .with_owner("rNFTOwner222".into())
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(ticket_offer.common_fields.ticket_sequence, Some(12345));
        assert_eq!(ticket_offer.owner.as_ref().unwrap(), "rNFTOwner222");
        // When using tickets, sequence should be None or 0
        assert!(ticket_offer.common_fields.sequence.is_none());
    }

    #[test]
    fn test_default() {
        let nftoken_create_offer = NFTokenCreateOffer {
            common_fields: CommonFields {
                account: "rTestAccount".into(),
                transaction_type: TransactionType::NFTokenCreateOffer,
                ..Default::default()
            },
            nftoken_id: "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007".into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        };

        assert_eq!(nftoken_create_offer.common_fields.account, "rTestAccount");
        assert_eq!(
            nftoken_create_offer.common_fields.transaction_type,
            TransactionType::NFTokenCreateOffer
        );
        assert_eq!(
            nftoken_create_offer.nftoken_id,
            "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007"
        );
        assert!(nftoken_create_offer.owner.is_none());
        assert!(nftoken_create_offer.expiration.is_none());
        assert!(nftoken_create_offer.destination.is_none());
    }
}
