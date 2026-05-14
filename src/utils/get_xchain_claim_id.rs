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

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_with_xchain_claim() -> TransactionMetadata<'static> {
        // Note: `Fields` derives `rename_all = "PascalCase"`, so
        // `xchain_claim_id` deserializes from `XchainClaimId` (not the
        // canonical XRPL casing `XChainClaimID`). Tracked separately.
        let json = r#"{
            "AffectedNodes": [
                {
                    "CreatedNode": {
                        "LedgerEntryType": "XChainOwnedClaimID",
                        "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                        "NewFields": {
                            "Account": "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe",
                            "Flags": 0,
                            "XchainClaimId": "13f"
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
    fn test_get_xchain_claim_id_success() {
        let meta = meta_with_xchain_claim();
        let id = get_xchain_claim_id(&meta).unwrap();
        assert_eq!(id, "13f");
    }

    #[test]
    fn test_get_xchain_claim_id_no_xchain_node() {
        let json = r#"{
            "AffectedNodes": [
                {
                    "CreatedNode": {
                        "LedgerEntryType": "AccountRoot",
                        "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                        "NewFields": {
                            "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                            "Flags": 0
                        }
                    }
                }
            ],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        let result = get_xchain_claim_id(&meta);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_xchain_claim_id_empty_affected_nodes() {
        let json = r#"{
            "AffectedNodes": [],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        let result = get_xchain_claim_id(&meta);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_xchain_claim_id_modified_node_only() {
        // Modified node of XChainOwnedClaimID should not match - only created
        // nodes are considered.
        let json = r#"{
            "AffectedNodes": [
                {
                    "ModifiedNode": {
                        "LedgerEntryType": "XChainOwnedClaimID",
                        "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                        "FinalFields": {
                            "Account": "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe",
                            "Flags": 0
                        }
                    }
                }
            ],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        let result = get_xchain_claim_id(&meta);
        assert!(result.is_err());
    }
}
