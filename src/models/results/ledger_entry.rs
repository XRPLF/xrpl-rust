use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::ledger::objects::LedgerEntry as LedgerObject;

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
    /// The data of this ledger entry, typed according to its `LedgerEntryType`.
    /// Omitted if "binary": true specified.
    pub node: Option<LedgerObject<'a>>,
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

        match result.node.unwrap() {
            LedgerObject::AccountRoot(account_root) => {
                assert_eq!(account_root.account, "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn");
                assert_eq!(
                    account_root.account_txn_id.as_deref(),
                    Some("4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB")
                );
                assert_eq!(account_root.balance.unwrap().0, "424021949");
                assert_eq!(account_root.domain.as_deref(), Some("6D64756F31332E636F6D"));
                assert_eq!(
                    account_root.email_hash.as_deref(),
                    Some("98B4375E1D753E5B91627516F6D70977")
                );
                assert_eq!(
                    account_root.message_key.as_deref(),
                    Some("0000000000000000000000070000000300")
                );
                assert_eq!(account_root.owner_count, 12);
                assert_eq!(
                    account_root.previous_txn_id,
                    "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB"
                );
                assert_eq!(account_root.previous_txn_lgr_seq, 61965653);
                assert_eq!(
                    account_root.regular_key.as_deref(),
                    Some("rD9iJmieYHn8jTtPjwwkW2Wm9sVDvPXLoJ")
                );
                assert_eq!(account_root.sequence, 385);
                assert_eq!(account_root.transfer_rate, Some(4294967295));
            }
            other => panic!("expected AccountRoot, got {other:?}"),
        }
    }

    #[test]
    fn test_ledger_entry_round_trip() {
        use crate::models::amount::XRPAmount;
        use crate::models::ledger::objects::account_root::AccountRoot;
        use crate::models::FlagCollection;

        let account_root = AccountRoot::new(
            FlagCollection::default(),
            Some("13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8".into()),
            None,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            12,
            "4E0AA11CBDD1760DE95B68DF2ABBE75C9698CEB548BEA9789053FCB3EBD444FB".into(),
            61965653,
            385,
            None,
            Some(XRPAmount::from("424021949")),
            None,
            Some("6D64756F31332E636F6D".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let entry = LedgerEntry {
            index: "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8".into(),
            ledger_index: Some(61966146),
            ledger_hash: Some(
                "31850E8E48E76D1064651DF39DF4E9542E8C90A9A9B629F4DE339EB3FA74F726".into(),
            ),
            node: Some(LedgerObject::AccountRoot(account_root)),
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
        // Non-AccountRoot entry: this is the shape that used to fail before
        // `node` was widened past a hardcoded AccountRoot-only struct (#308).
        let json = r#"{
            "index": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60",
            "ledger_index": 100,
            "node": {
                "Flags": 0,
                "Indexes": ["AAB0000000000000000000000000000000000000000000000000000000000"],
                "IndexNext": 0,
                "IndexPrevious": 0,
                "LedgerEntryType": "DirectoryNode",
                "Owner": "rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS",
                "RootIndex": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60",
                "index": "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60"
            },
            "validated": true
        }"#;

        let result: LedgerEntry = serde_json::from_str(json).unwrap();
        match result.node.unwrap() {
            LedgerObject::DirectoryNode(directory_node) => {
                assert_eq!(
                    directory_node.owner.as_deref(),
                    Some("rN7n3473SaZBCG4dFL83w7p1W9cgPLAPkS")
                );
                assert_eq!(
                    directory_node.root_index,
                    "A832B09498B80B1B1BB0E2B31B41B8A3A4B57B8C1C23DAF43A76C6B1B3F7CD60"
                );
            }
            other => panic!("expected DirectoryNode, got {other:?}"),
        }
    }
}
