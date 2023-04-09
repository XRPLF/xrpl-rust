use alloc::vec::Vec;
use anyhow::Result;
use core::convert::TryInto;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use alloc::string::ToString;

use crate::models::{
    model::Model,
    transactions::{Flag, Memo, Signer, Transaction, TransactionType},
};

use crate::Err;
use crate::_serde::txn_flags;
use crate::models::amount::exceptions::XRPLAmountException;
use crate::models::amount::{Amount, XRPAmount};
use crate::models::transactions::XRPLNFTokenCreateOfferException;

/// Transactions of the NFTokenCreateOffer type support additional values
/// in the Flags field. This enum represents those options.
///
/// See NFTokenCreateOffer flags:
/// `<https://xrpl.org/nftokencreateoffer.html#nftokencreateoffer-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
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
    #[serde(default = "TransactionType::nftoken_create_offer")]
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
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<NFTokenCreateOfferFlag>>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo<'a>>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the NFTokenCreateOffer model.
    ///
    /// See NFTokenCreateOffer fields:
    /// `<https://xrpl.org/nftokencreateoffer.html#nftokencreateoffer-fields>`
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: &'a str,
    pub amount: Amount<'a>,
    pub owner: Option<&'a str>,
    pub expiration: Option<u32>,
    pub destination: Option<&'a str>,
}

impl<'a> Default for NFTokenCreateOffer<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::NFTokenCreateOffer,
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
            nftoken_id: Default::default(),
            amount: Default::default(),
            owner: Default::default(),
            expiration: Default::default(),
            destination: Default::default(),
        }
    }
}

impl<'a: 'static> Model for NFTokenCreateOffer<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_amount_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => match self._get_destination_error() {
                Err(error) => Err!(error),
                Ok(_no_error) => match self._get_owner_error() {
                    Err(error) => Err!(error),
                    Ok(_no_error) => Ok(()),
                },
            },
        }
    }
}

impl<'a> Transaction for NFTokenCreateOffer<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        let mut flags = &Vec::new();

        if let Some(flag_set) = self.flags.as_ref() {
            flags = flag_set;
        }

        match flag {
            Flag::NFTokenCreateOffer(nftoken_create_offer_flag) => {
                match nftoken_create_offer_flag {
                    NFTokenCreateOfferFlag::TfSellOffer => {
                        flags.contains(&NFTokenCreateOfferFlag::TfSellOffer)
                    }
                }
            }
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }
}

impl<'a> NFTokenCreateOfferError for NFTokenCreateOffer<'a> {
    fn _get_amount_error(&self) -> Result<()> {
        let amount_into_decimal: Result<Decimal, XRPLAmountException> =
            self.amount.clone().try_into();
        match amount_into_decimal {
            Ok(amount) => {
                if !self.has_flag(&Flag::NFTokenCreateOffer(
                    NFTokenCreateOfferFlag::TfSellOffer,
                )) && amount.is_zero()
                {
                    Err!(XRPLNFTokenCreateOfferException::ValueZero {
                        field: "amount",
                        resource: "",
                    })
                } else {
                    Ok(())
                }
            }
            Err(decimal_error) => {
                Err!(decimal_error)
            }
        }
    }

    fn _get_destination_error(&self) -> Result<(), XRPLNFTokenCreateOfferException> {
        if let Some(destination) = self.destination {
            if destination == self.account {
                Err(XRPLNFTokenCreateOfferException::ValueEqualsValue {
                    field1: "destination",
                    field2: "account",
                    resource: "",
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_owner_error(&self) -> Result<(), XRPLNFTokenCreateOfferException> {
        if let Some(owner) = self.owner {
            if self.has_flag(&Flag::NFTokenCreateOffer(
                NFTokenCreateOfferFlag::TfSellOffer,
            )) {
                Err(XRPLNFTokenCreateOfferException::IllegalOption {
                    field: "owner",
                    context: "NFToken sell offers",
                    resource: "",
                })
            } else if owner == self.account {
                Err(XRPLNFTokenCreateOfferException::ValueEqualsValue {
                    field1: "owner",
                    field2: "account",
                    resource: "",
                })
            } else {
                Ok(())
            }
        } else if !self.has_flag(&Flag::NFTokenCreateOffer(
            NFTokenCreateOfferFlag::TfSellOffer,
        )) {
            Err(XRPLNFTokenCreateOfferException::OptionRequired {
                field: "owner",
                context: "NFToken buy offers",
                resource: "",
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> NFTokenCreateOffer<'a> {
    fn new(
        account: &'a str,
        nftoken_id: &'a str,
        amount: Amount<'a>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<&'a str>,
        signing_pub_key: Option<&'a str>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<&'a str>,
        flags: Option<Vec<NFTokenCreateOfferFlag>>,
        memos: Option<Vec<Memo<'a>>>,
        signers: Option<Vec<Signer<'a>>>,
        owner: Option<&'a str>,
        expiration: Option<u32>,
        destination: Option<&'a str>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::NFTokenCreateOffer,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags,
            memos,
            signers,
            nftoken_id,
            amount,
            owner,
            expiration,
            destination,
        }
    }
}

pub trait NFTokenCreateOfferError {
    fn _get_amount_error(&self) -> Result<()>;
    fn _get_destination_error(&self) -> Result<(), XRPLNFTokenCreateOfferException>;
    fn _get_owner_error(&self) -> Result<(), XRPLNFTokenCreateOfferException>;
}

#[cfg(test)]
mod test_nftoken_create_offer_error {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::models::{
        amount::{Amount, XRPAmount},
        Model,
    };

    use super::*;

    #[test]
    fn test_amount_error() {
        let nftoken_create_offer = NFTokenCreateOffer {
            transaction_type: TransactionType::NFTokenCreateOffer,
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
            nftoken_id: "",
            amount: Amount::XRPAmount(XRPAmount::from("0")),
            owner: None,
            expiration: None,
            destination: None,
        };

        assert_eq!(
            nftoken_create_offer
                .validate()
                .unwrap_err()
                .to_string()
                .as_str(),
            "The value of the field `amount` is not allowed to be zero. For more information see: "
        );
    }

    #[test]
    fn test_destination_error() {
        let nftoken_create_offer = NFTokenCreateOffer {
            transaction_type: TransactionType::NFTokenCreateOffer,
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
            nftoken_id: "",
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            owner: None,
            expiration: None,
            destination: Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb"),
        };

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The value of the field `destination` is not allowed to be the same as the value of the field `account`. For more information see: "
        );
    }

    #[test]
    fn test_owner_error() {
        let mut nftoken_create_offer = NFTokenCreateOffer {
            transaction_type: TransactionType::NFTokenCreateOffer,
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
            nftoken_id: "",
            amount: Amount::XRPAmount(XRPAmount::from("1")),
            owner: Some("rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK"),
            expiration: None,
            destination: None,
        };
        let sell_flag = vec![NFTokenCreateOfferFlag::TfSellOffer];
        nftoken_create_offer.flags = Some(sell_flag);

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The optional field `owner` is not allowed to be defined for NFToken sell offers. For more information see: "
        );

        nftoken_create_offer.flags = None;
        nftoken_create_offer.owner = None;

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The optional field `owner` is required to be defined for NFToken buy offers. For more information see: "
        );

        nftoken_create_offer.owner = Some("rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb");

        assert_eq!(
            nftoken_create_offer.validate().unwrap_err().to_string().as_str(),
            "The value of the field `owner` is not allowed to be the same as the value of the field `account`. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::amount::XRPAmount;
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = NFTokenCreateOffer::new(
            "rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX",
            "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007",
            Amount::XRPAmount(XRPAmount::from("1000000")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![NFTokenCreateOfferFlag::TfSellOffer]),
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"NFTokenCreateOffer","Account":"rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX","Flags":1,"NFTokenID":"000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007","Amount":"1000000"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = NFTokenCreateOffer::new(
            "rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX",
            "000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007",
            Amount::XRPAmount(XRPAmount::from("1000000")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![NFTokenCreateOfferFlag::TfSellOffer]),
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"NFTokenCreateOffer","Account":"rs8jBmmfpwgmrSPgwMsh7CvKRmRt1JTVSX","NFTokenID":"000100001E962F495F07A990F4ED55ACCFEEF365DBAA76B6A048C0A200000007","Amount":"1000000","Flags":1}"#;

        let txn_as_obj: NFTokenCreateOffer = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
