use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::{ledger::LedgerEntryType, NoFlags};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

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
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the DirectoryNode model.
    //
    // See DirectoryNode fields:
    // `<https://xrpl.org/directorynode.html#directorynode-fields>`
    /// (`Offer` Directories only) DEPRECATED. Do not use.
    pub exchange_rate: Option<Cow<'a, str>>,
    /// The contents of this `Directory`: an array of IDs of other objects.
    pub indexes: Vec<Cow<'a, str>>,
    /// The ID of root object for this directory.
    pub root_index: Cow<'a, str>,
    /// If this `Directory` consists of multiple pages, this ID links to the next object in the chain,
    /// wrapping around at the end.
    pub index_next: Option<u64>,
    /// If this `Directory` consists of multiple pages, this ID links to the previous object in the
    /// chain, wrapping around at the beginning.
    pub index_previous: Option<u64>,
    /// (Owner Directories only) The address of the account that owns the objects in this directory.
    pub owner: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The currency code of the `TakerGets` amount from the offers in this
    /// directory.
    pub taker_gets_currency: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The currency code of the `TakerPays` amount from the offers in this
    /// (`Offer` `Directories` only) The issuer of the `TakerGets` amount from the offers in this
    /// directory.
    pub taker_gets_issuer: Option<Cow<'a, str>>,
    /// directory.
    pub taker_pays_currency: Option<Cow<'a, str>>,
    /// (`Offer` `Directories` only) The issuer of the `TakerPays` amount from the offers in this
    /// directory.
    pub taker_pays_issuer: Option<Cow<'a, str>>,
}

impl<'a> Model for DirectoryNode<'a> {}

impl<'a> LedgerObject<NoFlags> for DirectoryNode<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> DirectoryNode<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
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
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::DirectoryNode,
                index,
                ledger_index,
            },
            exchange_rate,
            indexes,
            root_index,
            index_next,
            index_previous,
            owner,
            taker_gets_currency,
            taker_gets_issuer,
            taker_pays_currency,
            taker_pays_issuer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let directory_node = DirectoryNode::new(
            Some(Cow::from(
                "1BBEF97EDE88D40CEE2ADE6FEF121166AFE80D99EBADB01A4F069BA8FF484000",
            )),
            None,
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
        let serialized = serde_json::to_string(&directory_node).unwrap();

        let deserialized: DirectoryNode = serde_json::from_str(&serialized).unwrap();

        assert_eq!(directory_node, deserialized);
    }
}
