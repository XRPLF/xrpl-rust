use alloc::vec::Vec;

use crate::models::{
    ledger::objects::LedgerEntryType,
    requests::LedgerIndex,
    transactions::metadata::{AffectedNode, Fields, NodeType, TransactionMetadata},
};

#[derive(Debug)]
pub struct NormalizedNode<'a> {
    pub node_type: NodeType,
    pub ledger_entry_type: LedgerEntryType,
    pub ledger_index: LedgerIndex<'a>,
    pub new_fields: Option<Fields<'a>>,
    pub final_fields: Option<Fields<'a>>,
    pub previous_fields: Option<Fields<'a>>,
    pub previous_txn_id: Option<&'a str>,
    pub previous_txn_lgr_seq: Option<u32>,
}

fn normalize_node<'a: 'b, 'b>(affected_node: &'a AffectedNode<'_>) -> NormalizedNode<'b> {
    match affected_node {
        AffectedNode::CreatedNode {
            ledger_entry_type,
            ledger_index,
            new_fields,
        } => NormalizedNode {
            node_type: NodeType::CreatedNode,
            ledger_entry_type: ledger_entry_type.clone(),
            ledger_index: ledger_index.clone(),
            new_fields: Some(new_fields.clone()),
            final_fields: None,
            previous_fields: None,
            previous_txn_id: None,
            previous_txn_lgr_seq: None,
        },
        AffectedNode::ModifiedNode {
            ledger_entry_type,
            ledger_index,
            final_fields,
            previous_fields,
            previous_txn_id,
            previous_txn_lgr_seq,
        } => NormalizedNode {
            node_type: NodeType::ModifiedNode,
            ledger_entry_type: ledger_entry_type.clone(),
            ledger_index: ledger_index.clone(),
            new_fields: None,
            final_fields: final_fields.clone(),
            previous_fields: previous_fields.clone(),
            previous_txn_id: previous_txn_id.as_deref(),
            previous_txn_lgr_seq: *previous_txn_lgr_seq,
        },
        AffectedNode::DeletedNode {
            ledger_entry_type,
            ledger_index,
            final_fields,
            previous_fields,
        } => NormalizedNode {
            node_type: NodeType::DeletedNode,
            ledger_entry_type: ledger_entry_type.clone(),
            ledger_index: ledger_index.clone(),
            new_fields: None,
            final_fields: Some(final_fields.clone()),
            previous_fields: previous_fields.clone(),
            previous_txn_id: None,
            previous_txn_lgr_seq: None,
        },
    }
}

pub fn normalize_nodes<'a: 'b, 'b>(meta: &'a TransactionMetadata<'_>) -> Vec<NormalizedNode<'b>> {
    meta.affected_nodes
        .iter()
        .map(|node| normalize_node(node))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> TransactionMetadata<'static> {
        let json = r#"{
            "AffectedNodes": [
                {
                    "CreatedNode": {
                        "LedgerEntryType": "AccountRoot",
                        "LedgerIndex": "AAAA",
                        "NewFields": {
                            "Account": "rNew",
                            "Balance": "1000"
                        }
                    }
                },
                {
                    "ModifiedNode": {
                        "LedgerEntryType": "RippleState",
                        "LedgerIndex": "BBBB",
                        "FinalFields": {
                            "Account": "rMod",
                            "Balance": "2000"
                        },
                        "PreviousFields": {
                            "Balance": "1500"
                        },
                        "PreviousTxnId": "TXNID",
                        "PreviousTxnLgrSeq": 42
                    }
                },
                {
                    "DeletedNode": {
                        "LedgerEntryType": "Offer",
                        "LedgerIndex": "CCCC",
                        "FinalFields": {
                            "Account": "rDel",
                            "Balance": "0"
                        }
                    }
                }
            ],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_normalize_nodes_all_variants() {
        let meta = meta();
        let normalized = normalize_nodes(&meta);
        assert_eq!(normalized.len(), 3);

        // CreatedNode
        assert_eq!(normalized[0].node_type, NodeType::CreatedNode);
        assert_eq!(normalized[0].ledger_entry_type, LedgerEntryType::AccountRoot);
        assert!(normalized[0].new_fields.is_some());
        assert!(normalized[0].final_fields.is_none());
        assert!(normalized[0].previous_fields.is_none());

        // ModifiedNode
        assert_eq!(normalized[1].node_type, NodeType::ModifiedNode);
        assert_eq!(normalized[1].ledger_entry_type, LedgerEntryType::RippleState);
        assert!(normalized[1].new_fields.is_none());
        assert!(normalized[1].final_fields.is_some());
        assert!(normalized[1].previous_fields.is_some());
        assert_eq!(normalized[1].previous_txn_id, Some("TXNID"));
        assert_eq!(normalized[1].previous_txn_lgr_seq, Some(42));

        // DeletedNode
        assert_eq!(normalized[2].node_type, NodeType::DeletedNode);
        assert_eq!(normalized[2].ledger_entry_type, LedgerEntryType::Offer);
        assert!(normalized[2].new_fields.is_none());
        assert!(normalized[2].final_fields.is_some());
    }

    #[test]
    fn test_normalize_nodes_empty() {
        let meta: TransactionMetadata = serde_json::from_str(
            r#"{"AffectedNodes":[],"TransactionIndex":0,"TransactionResult":"tesSUCCESS"}"#,
        )
        .unwrap();
        let normalized = normalize_nodes(&meta);
        assert_eq!(normalized.len(), 0);
    }
}
