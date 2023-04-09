use crate::_serde::lgr_obj_flags;
use crate::models::ledger::LedgerEntryType;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum NFTokenOfferFlag {
    /// If enabled, the `NFTokenOffer` is a sell offer. Otherwise, the `NFTokenOffer` is a buy offer.
    LsfSellNFToken = 0x00000001,
}

/// The `NFTokenOffer` object represents an offer to buy, sell or transfer an `NFToken` object.
/// The owner of a `NFToken` can use `NFTokenCreateOffer` to start a transaction.
///
/// `<https://xrpl.org/nftokenoffer.html#nftokenoffer>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenOffer<'a> {
    /// The value `0x0037`, mapped to the string `NFTokenOffer`, indicates that this is an offer
    /// to trade a `NFToken`.
    pub ledger_entry_type: LedgerEntryType,
    /// A set of flags associated with this object, used to specify various options or settings.
    #[serde(with = "lgr_obj_flags")]
    pub flags: Vec<NFTokenOfferFlag>,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// Amount expected or offered for the `NFToken`. If the token has the `lsfOnlyXRP` flag set,
    /// the amount must be specified in XRP. Sell offers that specify assets other than XRP
    /// must specify a non-zero amount. Sell offers that specify XRP can be 'free'
    /// (that is, the Amount field can be equal to "0").
    pub amount: Amount<'a>,
    /// The `NFTokenID` of the `NFToken` object referenced by this offer.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    /// Owner of the account that is creating and owns the offer. Only the current Owner
    /// of an `NFToken` can create an offer to sell an `NFToken`, but any account can create
    /// an offer to buy an NFToken.
    pub owner: Cow<'a, str>,
    /// Identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// Index of the ledger that contains the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The `AccountID` for which this offer is intended. If present, only that account can
    /// accept the offer.
    pub destination: Option<Cow<'a, str>>,
    /// The time after which the offer is no longer active. The value is the number of
    /// seconds since the Ripple Epoch.
    pub expiration: Option<u32>,
    /// Internal bookkeeping, indicating the page inside the token buy or sell offer directory,
    /// as appropriate, where this token is being tracked. This field allows the efficient
    /// deletion of offers.
    #[serde(rename = "NFTokenOfferNode")]
    pub nftoken_offer_node: Option<Cow<'a, str>>,
    /// Internal bookkeeping, indicating the page inside the owner directory where this token
    /// is being tracked. This field allows the efficient deletion of offers.
    pub owner_node: Option<Cow<'a, str>>,
}

impl<'a> Default for NFTokenOffer<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NFTokenOffer,
            flags: Default::default(),
            index: Default::default(),
            amount: Default::default(),
            nftoken_id: Default::default(),
            owner: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
            destination: Default::default(),
            expiration: Default::default(),
            nftoken_offer_node: Default::default(),
            owner_node: Default::default(),
        }
    }
}

impl<'a> Model for NFTokenOffer<'a> {}

impl<'a> NFTokenOffer<'a> {
    pub fn new(
        flags: Vec<NFTokenOfferFlag>,
        index: Cow<'a, str>,
        amount: Amount<'a>,
        nftoken_id: Cow<'a, str>,
        owner: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        destination: Option<Cow<'a, str>>,
        expiration: Option<u32>,
        nftoken_offer_node: Option<Cow<'a, str>>,
        owner_node: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::NFTokenOffer,
            flags,
            index,
            amount,
            nftoken_id,
            owner,
            previous_txn_id,
            previous_txn_lgr_seq,
            destination,
            expiration,
            nftoken_offer_node,
            owner_node,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::vec;

    #[test]
    fn test_serialization() {
        let nftoken_offer = NFTokenOffer::new(
            vec![NFTokenOfferFlag::LsfSellNFToken],
            Cow::from("AEBABA4FAC212BF28E0F9A9C3788A47B085557EC5D1429E7A8266FB859C863B3"),
            Amount::XRPAmount("1000000".into()),
            Cow::from("00081B5825A08C22787716FA031B432EBBC1B101BB54875F0002D2A400000000"),
            Cow::from("rhRxL3MNvuKEjWjL7TBbZSDacb8PmzAd7m"),
            Cow::from("BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769"),
            75443565,
            None,
            None,
            Some(Cow::from("0")),
            Some(Cow::from("17")),
        );
        let nftoken_offer_json = serde_json::to_string(&nftoken_offer).unwrap();
        let actual = nftoken_offer_json.as_str();
        let expected = r#"{"LedgerEntryType":"NFTokenOffer","Flags":1,"index":"AEBABA4FAC212BF28E0F9A9C3788A47B085557EC5D1429E7A8266FB859C863B3","Amount":"1000000","NFTokenID":"00081B5825A08C22787716FA031B432EBBC1B101BB54875F0002D2A400000000","Owner":"rhRxL3MNvuKEjWjL7TBbZSDacb8PmzAd7m","PreviousTxnID":"BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769","PreviousTxnLgrSeq":75443565,"NFTokenOfferNode":"0","OwnerNode":"17"}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
