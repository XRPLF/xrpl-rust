use alloc::vec::Vec;

use crate::models::{
    ledger::objects::LedgerEntryType,
    requests::LedgerIndex,
    transactions::metadata::{AffectedNode, Fields, NodeType, TransactionMetadata},
};

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
