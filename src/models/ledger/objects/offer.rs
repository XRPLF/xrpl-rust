use crate::_serde::lgr_obj_flags;
use crate::models::ledger::LedgerEntryType;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{AsRefStr, Display, EnumIter};

use serde_with::skip_serializing_none;

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum OfferFlag {
    /// The object was placed as a passive Offer.
    LsfPassive = 0x00010000,
    /// The object was placed as a sell Offer.
    LsfSell = 0x00020000,
}

/// The Offer ledger entry describes an Offer to exchange currencies in the XRP Ledger's
/// decentralized exchange. (In finance, this is more traditionally known as an order.)
/// An OfferCreate transaction only creates an Offer entry in the ledger when the Offer
/// cannot be fully executed immediately by consuming other Offers already in the ledger.
///
/// `<https://xrpl.org/offer.html#offer>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Offer<'a> {
    /// The value `0x006F`, mapped to the string `Offer`, indicates that this object
    /// describes an `Offer`.
    ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this offer.
    #[serde(with = "lgr_obj_flags")]
    flags: Vec<OfferFlag>,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The address of the account that owns this `Offer`.
    pub account: Cow<'a, str>,
    /// The ID of the `Offer Directory` that links to this Offer.
    pub book_directory: Cow<'a, str>,
    /// A hint indicating which page of the offer directory links to this object, in case
    /// the directory consists of multiple pages.
    pub book_node: Cow<'a, str>,
    /// A hint indicating which page of the owner directory links to this object, in case
    /// the directory consists of multiple pages.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified
    /// this object.
    pub previous_txn_lgr_seq: u32,
    /// The `Sequence` value of the `OfferCreate` transaction that created this `Offer` object.
    /// Used in combination with the `Account` to identify this `Offer`.
    pub sequence: u32,
    /// The remaining amount and type of currency being provided by the `Offer` creator.
    pub taker_gets: Amount<'a>,
    /// The remaining amount and type of currency requested by the `Offer` creator.
    pub taker_pays: Amount<'a>,
    /// Indicates the time after which this Offer is considered unfunded.
    pub expiration: Option<u32>,
}

impl<'a> Default for Offer<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Offer,
            flags: Default::default(),
            index: Default::default(),
            account: Default::default(),
            book_directory: Default::default(),
            book_node: Default::default(),
            owner_node: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
            sequence: Default::default(),
            taker_gets: Default::default(),
            taker_pays: Default::default(),
            expiration: Default::default(),
        }
    }
}

impl<'a> Model for Offer<'a> {}

impl<'a> Offer<'a> {
    pub fn new(
        flags: Vec<OfferFlag>,
        index: Cow<'a, str>,
        account: Cow<'a, str>,
        book_directory: Cow<'a, str>,
        book_node: Cow<'a, str>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        sequence: u32,
        taker_gets: Amount<'a>,
        taker_pays: Amount<'a>,
        expiration: Option<u32>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::Offer,
            flags,
            index,
            account,
            book_directory,
            book_node,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            sequence,
            taker_gets,
            taker_pays,
            expiration,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use crate::models::amount::IssuedCurrencyAmount;
    use alloc::borrow::Cow;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let offer = Offer::new(
            vec![OfferFlag::LsfSell],
            Cow::from("96F76F27D8A327FC48753167EC04A46AA0E382E6F57F32FD12274144D00F1797"),
            Cow::from("rBqb89MRQJnMPq8wTwEbtz4kvxrEDfcYvt"),
            Cow::from("ACC27DE91DBA86FC509069EAF4BC511D73128B780F2E54BF5E07A369E2446000"),
            Cow::from("0000000000000000"),
            Cow::from("0000000000000000"),
            Cow::from("F0AB71E777B2DA54B86231E19B82554EF1F8211F92ECA473121C655BFC5329BF"),
            14524914,
            866,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "XAG".into(),
                "r9Dr5xwkeLegBeXq6ujinjSBLQzQ1zQGjH".into(),
                "37".into(),
            )),
            Amount::XRPAmount("79550000000".into()),
            None,
        );
        let offer_json = serde_json::to_string(&offer).unwrap();
        let actual = offer_json.as_str();
        let expected = r#"{"LedgerEntryType":"Offer","Flags":131072,"index":"96F76F27D8A327FC48753167EC04A46AA0E382E6F57F32FD12274144D00F1797","Account":"rBqb89MRQJnMPq8wTwEbtz4kvxrEDfcYvt","BookDirectory":"ACC27DE91DBA86FC509069EAF4BC511D73128B780F2E54BF5E07A369E2446000","BookNode":"0000000000000000","OwnerNode":"0000000000000000","PreviousTxnID":"F0AB71E777B2DA54B86231E19B82554EF1F8211F92ECA473121C655BFC5329BF","PreviousTxnLgrSeq":14524914,"Sequence":866,"TakerGets":{"currency":"XAG","issuer":"r9Dr5xwkeLegBeXq6ujinjSBLQzQ1zQGjH","value":"37"},"TakerPays":"79550000000"}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
