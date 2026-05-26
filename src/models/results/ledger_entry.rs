use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response format for the ledger_entry method, which returns a single ledger
/// object from the XRP Ledger in its raw format.
///
/// See Ledger Entry:
/// `<https://xrpl.org/ledger_entry.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LedgerEntry<'a> {
    /// The unique ID of this ledger entry.
    pub index: Cow<'a, str>,
    /// The ledger index of the ledger that was used when retrieving this data.
    pub ledger_index: Option<u32>,
    /// The identifying hash of the ledger version used to retrieve this data
    pub ledger_hash: Option<Cow<'a, str>>,
    /// Object containing the data of this ledger entry, according to the
    /// ledger format. Omitted if "binary": true specified.
    /// This is a generic JSON value because `ledger_entry` can return any
    /// ledger object type (AccountRoot, DirectoryNode, Offer, etc.).
    pub node: Option<Value>,
    /// The binary representation of the ledger object, as hexadecimal.
    /// Only present if "binary": true specified.
    pub node_binary: Option<Cow<'a, str>>,
    /// (Clio server only) The ledger index where the ledger entry object was
    /// deleted. Only present if include_deleted parameter is set.
    pub deleted_ledger_index: Option<Cow<'a, str>>,
    /// Whether this data is from a validated ledger version
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_entry_deserialize() {
        let json = r#"{
            "index": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8",
            "ledger_hash": "31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726",
            "ledger_index": 61966146,
            "node": {
                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                "AccountTxnID": "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB",
                "Balance": "424021949",
                "Domain": "6D64756F31332E636F6D",
                "EmailHash": "98B4375E1D753E5B91627516F6D70977",
                "Flags": 9568256,
                "LedgerEntryType": "AccountRoot",
                "MessageKey": "0000000000000000000000070000000300",
                "OwnerCount": 12,
                "PreviousTxnID": "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB",
                "PreviousTxnLgrSeq": 61965653,
                "RegularKey": "rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ",
                "Sequence": 385,
                "TransferRate": 4294967295,
                "index": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
            },
            "validated": true
        }"#;

        let result: LedgerEntry = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.index,
            "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
        );
        assert_eq!(result.ledger_index, Some(61966146));
        assert_eq!(
            result.ledger_hash,
            Some("31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726".into())
        );
        assert_eq!(result.validated, Some(true));

        let node = result.node.unwrap();
        assert_eq!(node["Account"], "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
        assert_eq!(
            node["AccountTxnID"],
            "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB"
        );
        assert_eq!(node["Balance"], "424021949");
        assert_eq!(node["Domain"], "6D64756F31332E636F6D");
        assert_eq!(node["EmailHash"], "98B4375E1D753E5B91627516F6D70977");
        assert_eq!(node["Flags"], 9568256);
        assert_eq!(node["LedgerEntryType"], "AccountRoot");
        assert_eq!(node["MessageKey"], "0000000000000000000000070000000300");
        assert_eq!(node["OwnerCount"], 12);
        assert_eq!(
            node["PreviousTxnID"],
            "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB"
        );
        assert_eq!(node["PreviousTxnLgrSeq"], 61965653);
        assert_eq!(node["RegularKey"], "rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ");
        assert_eq!(node["Sequence"], 385);
        assert_eq!(node["TransferRate"], 4294967295u64);
        assert_eq!(
            node["index"],
            "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
        );
    }

    #[test]
    fn test_ledger_entry_round_trip() {
        let entry = LedgerEntry {
            index: "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8".into(),
            ledger_index: Some(61966146),
            ledger_hash: Some(
                "31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726".into(),
            ),
            node: Some(serde_json::json!({
                "Account": "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn",
                "Balance": "424021949",
                "Domain": "6D64756F31332E636F6D",
                "Flags": 9568256,
                "LedgerEntryType": "AccountRoot",
                "OwnerCount": 12,
                "PreviousTxnID": "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB",
                "PreviousTxnLgrSeq": 61965653,
                "Sequence": 385,
                "index": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8"
            })),
            node_binary: None,
            deleted_ledger_index: None,
            validated: Some(true),
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_ledger_entry_default() {
        let entry: LedgerEntry = LedgerEntry::default();
        assert_eq!(entry.index, "");
        assert!(entry.node.is_none());
    }

    #[test]
    fn test_ledger_entry_node_binary_only() {
        let json = r#"{
            "index": "ABC",
            "ledger_index": 1,
            "node_binary": "AABBCC",
            "validated": false
        }"#;
        let entry: LedgerEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.node_binary.as_deref(), Some("AABBCC"));
        assert!(entry.node.is_none());
        assert_eq!(entry.validated, Some(false));
    }

    #[test]
    fn test_ledger_entry_directory_node() {
        let json = r#"{
            "index": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60",
            "ledger_index": 100,
            "node": {
                "Flags": 0,
                "Indexes": ["AAB..."],
                "IndexNext": "0",
                "IndexPrevious": "0",
                "LedgerEntryType": "DirectoryNode",
                "Owner": "rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS",
                "RootIndex": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60",
                "index": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60"
            },
            "validated": true
        }"#;

        let result: LedgerEntry = serde_json::from_str(json).unwrap();
        let node = result.node.unwrap();
        assert_eq!(node["LedgerEntryType"], "DirectoryNode");
        assert_eq!(node["Owner"], "rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS");
    }
}
