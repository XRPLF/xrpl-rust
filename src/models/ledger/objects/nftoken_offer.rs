use crate::models::ledger::LedgerEntryType;
use crate::models::FlagCollection;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use super::{CommonFields, LedgerObject};

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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NFTokenOfferFlag>,
    // The custom fields for the NFTokenOffer model.
    //
    // See NFTokenOffer fields:
    // `<https://xrpl.org/nftokenoffer.html#nftokenoffer-fields>`
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

impl<'a> Model for NFTokenOffer<'a> {}

impl<'a> LedgerObject<NFTokenOfferFlag> for NFTokenOffer<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> NFTokenOffer<'a> {
    pub fn new(
        flags: FlagCollection<NFTokenOfferFlag>,
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
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
            common_fields: CommonFields {
                flags,
                ledger_entry_type: LedgerEntryType::NFTokenOffer,
                index,
                ledger_index,
            },
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
mod tests {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let nftoken_offer = NFTokenOffer::new(
            vec![NFTokenOfferFlag::LsfSellNFToken].into(),
            Some(Cow::from(
                "AEBABA4FAC212BF28E0F9A9C3788A47B085557EC5D1429E7A8266FB859C863B3",
            )),
            None,
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
        let serialized = serde_json::to_string(&nftoken_offer).unwrap();

        let deserialized: NFTokenOffer = serde_json::from_str(&serialized).unwrap();

        assert_eq!(nftoken_offer, deserialized);
    }
}
