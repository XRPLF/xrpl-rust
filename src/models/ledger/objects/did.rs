use crate::models::ledger::objects::LedgerEntryType;
use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::NoFlags;
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `DID` object type holds references to, or data associated with, a single
/// Decentralized Identifier (DID).
///
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/did>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DID<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account that controls the DID.
    pub account: Cow<'a, str>,
    /// The W3C standard DID document associated with the DID.
    /// Limited to a maximum length of 256 bytes.
    #[serde(rename = "DIDDocument")]
    pub did_document: Option<Cow<'a, str>>,
    /// The public attestations of identity credentials associated with the DID.
    /// Limited to a maximum length of 256 bytes.
    pub data: Option<Cow<'a, str>>,
    /// The Universal Resource Identifier that points to the corresponding
    /// DID document or the data associated with the DID.
    /// Limited to a maximum length of 256 bytes.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
    /// A hint indicating which page of the owner directory links to this object.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for DID<'a> {}

impl<'a> LedgerObject<NoFlags> for DID<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> DID<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        did_document: Option<Cow<'a, str>>,
        data: Option<Cow<'a, str>>,
        uri: Option<Cow<'a, str>>,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::DID,
                index,
                ledger_index,
            },
            account,
            did_document,
            data,
            uri,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::borrow::Cow;

    #[test]
    fn test_serialize() {
        let did = DID::new(
            Some(Cow::from(
                "46813BE38B798B3752CA590D44E7FEADB17485649074403AD1761A2835CE91FF",
            )),
            None,
            Cow::from("rpfqJrXg5uidNo2ZsRhRY6TiF1cvYmV9Fg"),
            Some(Cow::from("646F63")),
            Some(Cow::from("617474657374")),
            Some(Cow::from("6469645F6578616D706C65")),
            Cow::from("0"),
            Cow::from("A4C15DA185E6092DF5954FF62A1446220C61A5F60F0D93B4B09F708778E41120"),
            4,
        );
        let serialized = serde_json::to_string(&did).unwrap();
        let deserialized: DID = serde_json::from_str(&serialized).unwrap();
        assert_eq!(did, deserialized);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{
            "Account": "rpfqJrXg5uidNo2ZsRhRY6TiF1cvYmV9Fg",
            "DIDDocument": "646F63",
            "Data": "617474657374",
            "Flags": 0,
            "LedgerEntryType": "DID",
            "OwnerNode": "0",
            "PreviousTxnID": "A4C15DA185E6092DF5954FF62A1446220C61A5F60F0D93B4B09F708778E41120",
            "PreviousTxnLgrSeq": 4,
            "URI": "6469645F6578616D706C65",
            "index": "46813BE38B798B3752CA590D44E7FEADB17485649074403AD1761A2835CE91FF"
        }"#;

        let did: DID = serde_json::from_str(json).unwrap();
        assert_eq!(did.account, "rpfqJrXg5uidNo2ZsRhRY6TiF1cvYmV9Fg");
        assert_eq!(did.did_document.as_deref(), Some("646F63"));
        assert_eq!(did.data.as_deref(), Some("617474657374"));
        assert_eq!(did.uri.as_deref(), Some("6469645F6578616D706C65"));
        assert_eq!(did.owner_node, "0");
        assert_eq!(did.previous_txn_lgr_seq, 4);
    }
}
