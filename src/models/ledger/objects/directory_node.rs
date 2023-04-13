use crate::models::ledger::LedgerEntryType;
use crate::models::Model;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

/// The `DirectoryNode` object type provides a list of links to other objects in the ledger's state
/// tree. A single conceptual Directory　takes the form of a doubly linked list, with one or more
/// `DirectoryNode` objects each containing up to 32 IDs of other objects. The first object is called
/// the root of the directory, and all objects other than the root object can be added or deleted
/// as necessary.
///
/// There are two kinds of Directories:
/// - `Owner` directories list other objects owned by an account, such as `RippleState` (trust line)
/// or `Offer` objects.
/// - `Offer` directories list the offers available in the decentralized exchange. A single `Offer`
/// directory contains all the offers that have the same exchange rate for the same token.
///
/// `<https://xrpl.org/directorynode.html#directorynode>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DirectoryNode<'a> {
    /// The value 0x0064, mapped to the string `DirectoryNode`, indicates that this object is part
    /// of a Directory.
    pub ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean flags enabled for this object. Currently, the protocol defines no flags
    /// for `DirectoryNode` objects. The value is always 0.
    pub flags: u32,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// (`Offer` Directories only) DEPRECATED. Do not use.
    pub exchange_rate: Option<Cow<'a, str>>,
    /// The contents of this `Directory`: an array of IDs of other objects.
    pub indexes: Vec<Cow<'a, str>>,
    /// If this `Directory` consists of multiple pages, this ID links to the next object in the chain,
    /// wrapping around at the end.
    pub index_next: Option<u64>,
    /// If this `Directory` consists of multiple pages, this ID links to the previous object in the
    /// chain, wrapping around at the beginning.
    pub index_previous: Option<u64>,
    /// (Owner Directories only) The address of the account that owns the objects in this directory.
    pub owner: Option<Cow<'a, str>>,
    /// The ID of root object for this directory.
    pub root_index: Cow<'a, str>,
    /// (`Offer` `Directories` only) The currency code of the `TakerGets` amount from the offers in this
    /// directory.
    pub taker_gets_currency: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The issuer of the `TakerGets` amount from the offers in this
    /// directory.
    pub taker_gets_issuer: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The currency code of the `TakerPays` amount from the offers in this
    /// directory.
    pub taker_pays_currency: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The issuer of the `TakerPays` amount from the offers in this
    /// directory.
    pub taker_pays_issuer: Option<Cow<'a, str>>,
}

impl<'a> Default for DirectoryNode<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::DirectoryNode,
            flags: Default::default(),
            index: Default::default(),
            exchange_rate: Default::default(),
            indexes: Default::default(),
            index_next: Default::default(),
            index_previous: Default::default(),
            owner: Default::default(),
            root_index: Default::default(),
            taker_gets_currency: Default::default(),
            taker_gets_issuer: Default::default(),
            taker_pays_currency: Default::default(),
            taker_pays_issuer: Default::default(),
        }
    }
}

impl<'a> Model for DirectoryNode<'a> {}

impl<'a> DirectoryNode<'a> {
    pub fn new(
        index: Cow<'a, str>,
        indexes: Vec<Cow<'a, str>>,
        root_index: Cow<'a, str>,
        exchange_rate: Option<Cow<'a, str>>,
        index_next: Option<u64>,
        index_previous: Option<u64>,
        owner: Option<Cow<'a, str>>,
        taker_gets_currency: Option<Cow<'a, str>>,
        taker_gets_issuer: Option<Cow<'a, str>>,
        taker_pays_currency: Option<Cow<'a, str>>,
        taker_pays_issuer: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::DirectoryNode,
            flags: 0,
            index,
            exchange_rate,
            indexes,
            index_next,
            index_previous,
            owner,
            root_index,
            taker_gets_currency,
            taker_gets_issuer,
            taker_pays_currency,
            taker_pays_issuer,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let directory_node = DirectoryNode::new(
            Cow::from("1BBEF97EDE88D40CEE2ADE6FEF121166AFE80D99EBADB01A4F069BA8FF484000"),
            vec![Cow::from(
                "AD7EAE148287EF12D213A251015F86E6D4BD34B3C4A0A1ED9A17198373F908AD",
            )],
            Cow::from("1BBEF97EDE88D40CEE2ADE6FEF121166AFE80D99EBADB01A4F069BA8FF484000"),
            Some(Cow::from("4F069BA8FF484000")),
            None,
            None,
            None,
            Some(Cow::from("0000000000000000000000000000000000000000")),
            Some(Cow::from("0000000000000000000000000000000000000000")),
            Some(Cow::from("0000000000000000000000004A50590000000000")),
            Some(Cow::from("5BBC0F22F61D9224A110650CFE21CC0C4BE13098")),
        );
        let directory_node_json = serde_json::to_string(&directory_node).unwrap();
        let actual = directory_node_json.as_str();
        let expected = r#"{"LedgerEntryType":"DirectoryNode","Flags":0,"index":"1BBEF97EDE88D40CEE2ADE6FEF121166AFE80D99EBADB01A4F069BA8FF484000","ExchangeRate":"4F069BA8FF484000","Indexes":["AD7EAE148287EF12D213A251015F86E6D4BD34B3C4A0A1ED9A17198373F908AD"],"RootIndex":"1BBEF97EDE88D40CEE2ADE6FEF121166AFE80D99EBADB01A4F069BA8FF484000","TakerGetsCurrency":"0000000000000000000000000000000000000000","TakerGetsIssuer":"0000000000000000000000000000000000000000","TakerPaysCurrency":"0000000000000000000000004A50590000000000","TakerPaysIssuer":"5BBC0F22F61D9224A110650CFE21CC0C4BE13098"}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
