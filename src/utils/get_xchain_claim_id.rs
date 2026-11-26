use alloc::borrow::Cow;

use crate::models::{
    ledger::objects::LedgerEntryType, transactions::metadata::TransactionMetadata,
};

use super::exceptions::{XRPLUtilsException, XRPLUtilsResult, XRPLXChainClaimIdException};
use crate::models::transactions::metadata::AffectedNode;

pub fn get_xchain_claim_id<'a: 'b, 'b>(
    meta: &TransactionMetadata<'a>,
) -> XRPLUtilsResult<Cow<'b, str>> {
    // `AffectedNodes` ordering is not guaranteed by the server, so search for
    // the created `XChainOwnedClaimID` entry by type and field rather than
    // trusting a positional index.
    meta.affected_nodes
        .iter()
        .find_map(|node| match node {
            AffectedNode::CreatedNode {
                ledger_entry_type,
                new_fields,
                ..
            } if ledger_entry_type == &LedgerEntryType::XChainOwnedClaimID => {
                new_fields.xchain_claim_id.clone()
            }
            _ => None,
        })
        .ok_or(XRPLUtilsException::XRPLXChainClaimIdError(
            XRPLXChainClaimIdException::NoXChainOwnedClaimID,
        ))
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
    fn test_get_xchain_claim_id_not_first_node() {
        // The XChainOwnedClaimID created node is not at index 0; the lookup
        // must still find it by type rather than trusting the ordering.
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
                },
                {
                    "CreatedNode": {
                        "LedgerEntryType": "XChainOwnedClaimID",
                        "LedgerIndex": "AA1ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                        "NewFields": {
                            "Account": "rPV4mZjsXfH2HvUSPLNmqz1J8d3Lpv7tpe",
                            "Flags": 0,
                            "XchainClaimId": "abc"
                        }
                    }
                }
            ],
            "TransactionIndex": 0,
            "TransactionResult": "tesSUCCESS"
        }"#;
        let meta: TransactionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(get_xchain_claim_id(&meta).unwrap(), "abc");
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
