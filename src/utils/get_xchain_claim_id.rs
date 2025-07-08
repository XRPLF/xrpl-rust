use alloc::{borrow::Cow, vec::Vec};

use crate::models::{
    ledger::objects::LedgerEntryType, transactions::metadata::TransactionMetadata,
};

use super::exceptions::{XRPLUtilsException, XRPLUtilsResult, XRPLXChainClaimIdException};
use crate::models::transactions::metadata::AffectedNode;

pub fn get_xchain_claim_id<'a: 'b, 'b>(
    meta: &TransactionMetadata<'a>,
) -> XRPLUtilsResult<Cow<'b, str>> {
    let affected_nodes: Vec<&AffectedNode> = meta
        .affected_nodes
        .iter()
        .filter(|node| {
            // node.is_created_node() && node.created_node().ledger_entry_type == "XChainOwnedClaimID"
            match node {
                AffectedNode::CreatedNode {
                    ledger_entry_type, ..
                } => ledger_entry_type == &LedgerEntryType::XChainOwnedClaimID,
                _ => false,
            }
        })
        .collect();

    if affected_nodes.is_empty() {
        Err(XRPLXChainClaimIdException::NoXChainOwnedClaimID.into())
    } else {
        match affected_nodes[0] {
            AffectedNode::CreatedNode { new_fields, .. } => {
                if let Some(xchain_claim_id) = new_fields.xchain_claim_id.as_ref() {
                    Ok(xchain_claim_id.clone())
                } else {
                    Err(XRPLUtilsException::XRPLXChainClaimIdError(
                        XRPLXChainClaimIdException::NoXChainOwnedClaimID,
                    ))
                }
            }
            _ => Err(XRPLUtilsException::XRPLXChainClaimIdError(
                XRPLXChainClaimIdException::NoXChainOwnedClaimID,
            )),
        }
    }
}
