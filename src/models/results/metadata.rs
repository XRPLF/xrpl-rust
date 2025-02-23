use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// See Metadata:
/// `<https://xrpl.org/docs/references/protocol/transactions/metadata>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMetadata<'a> {
    /// The transaction's position within the ledger that included it.
    #[serde(rename = "TransactionIndex")]
    pub transaction_index: u64,
    /// The transaction's result code.
    #[serde(rename = "TransactionResult")]
    pub transaction_result: Cow<'a, str>,
    /// Array of objects describing changes to ledger entries this
    /// transaction made.
    #[serde(rename = "AffectedNodes")]
    pub affected_nodes: Cow<'a, [AffectedNode<'a>]>,
    /// The currency amount actually delivered to the destination for Payment
    /// transactions. Contains "unavailable" for partial payments before
    /// 2014-01-20.
    pub delivered_amount: Option<Value>,
    /// (Optional) NFTokenID for NFTokenMint and NFTokenAcceptOffer
    /// transactions.
    #[serde(rename = "nftoken_id")]
    pub nftoken_id: Option<Cow<'a, str>>,
    /// (Optional) Array of NFTokenIDs for NFTokenCancelOffer transactions.
    #[serde(rename = "nftoken_ids")]
    pub nftoken_ids: Option<Cow<'a, [Cow<'a, str>]>>,
    /// (Optional) OfferID for NFTokenCreateOffer transactions.
    #[serde(rename = "offer_id")]
    pub offer_id: Option<Cow<'a, str>>,
    /// (Optional) MPTokenIssuanceID for MPTokenIssuanceCreate transactions.
    #[serde(rename = "mpt_issuance_id")]
    pub mpt_issuance_id: Option<Cow<'a, str>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedNode<'a> {
    #[serde(rename = "CreatedNode")]
    pub created_node: Option<LedgerNode<'a>>,
    #[serde(rename = "ModifiedNode")]
    pub modified_node: Option<LedgerNode<'a>>,
    #[serde(rename = "DeletedNode")]
    pub deleted_node: Option<LedgerNode<'a>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct LedgerNode<'a> {
    /// The type of ledger object this node represents.
    pub ledger_entry_type: Cow<'a, str>,
    /// The ID of this ledger entry in the ledger's state tree.
    pub ledger_index: Cow<'a, str>,
    /// The content fields of the ledger entry after changes.
    pub final_fields: Option<Value>,
    /// The previous values for changed fields.
    pub previous_fields: Option<Value>,
    /// The content fields of a newly created ledger entry.
    pub new_fields: Option<Value>,
    /// The identifying hash of the previous transaction to modify this
    /// ledger entry.
    pub previous_txn_id: Option<Cow<'a, str>>,
    /// The Ledger Index of the ledger containing the previous transaction.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: Option<u32>,
    /// The node in the directory chain.
    pub book_node: Option<Cow<'a, str>>,
    /// The node in the owner directory chain.
    pub owner_node: Option<Cow<'a, str>>,
    /// The exchange rate, used in offer directory nodes.
    pub exchange_rate: Option<Cow<'a, str>>,
    /// The root index of the directory.
    pub root_index: Option<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transaction_metadata_deserialize() {
        let json = r#"{
            "AffectedNodes": [
                {
                    "ModifiedNode": {
                        "FinalFields": {
                            "Account": "rBTwLga3i2gz3doX6Gva3MgEV8ZCD8jjah",
                            "Balance": "27724423128",
                            "Flags": 0,
                            "OwnerCount": 14,
                            "Sequence": 129693478
                        },
                        "LedgerEntryType": "AccountRoot",
                        "LedgerIndex": "1ED8DDFD80F275CB1CE7F18BB9D906655DE8029805D8B95FB9020B30425821EB",
                        "PreviousFields": {
                            "Balance": "27719423228",
                            "Sequence": 129693477
                        },
                        "PreviousTxnID": "3110F983CDC090750B45C9BFB74B8CE629CA80F57C35612402B2760153822BA5",
                        "PreviousTxnLgrSeq": 86724072
                    }
                },
                {
                    "DeletedNode": {
                        "FinalFields": {
                            "Account": "rPx6Rbh8fStXeP3LwECBisownN2ZyMyzYS",
                            "BookDirectory": "DFA3B6DDAB58C7E8E5D944E736DA4B7046C30E4F460FD9DE4E1566CBCC208000",
                            "BookNode": "0",
                            "Flags": 0,
                            "OwnerNode": "0",
                            "PreviousTxnID": "DCB061EC44BBF73BBC20CE0432E9D8D7C4B8B28ABA8AE5A5BA687476E7A796EF",
                            "PreviousTxnLgrSeq": 86724050,
                            "Sequence": 86586865,
                            "TakerGets": "0",
                            "TakerPays": {
                                "currency": "USD",
                                "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                                "value": "0"
                            }
                        },
                        "LedgerEntryType": "Offer",
                        "LedgerIndex": "348AF66EBD872FBF2BD23085D3FB4A200E15509451475027C4A5EE8D8B77C623"
                    }
                }
            ],
            "TransactionIndex": 5,
            "TransactionResult": "tesSUCCESS"
        }"#;

        let metadata: TransactionMetadata = serde_json::from_str(json).unwrap();

        assert_eq!(metadata.transaction_index, 5);
        assert_eq!(metadata.transaction_result, "tesSUCCESS");
        assert_eq!(metadata.affected_nodes.len(), 2);

        // Test first affected node (ModifiedNode)
        let first_node = &metadata.affected_nodes[0];
        if let Some(modified) = &first_node.modified_node {
            assert_eq!(modified.ledger_entry_type, "AccountRoot");
            assert_eq!(
                modified.ledger_index,
                "1ED8DDFD80F275CB1CE7F18BB9D906655DE8029805D8B95FB9020B30425821EB"
            );
            assert_eq!(modified.previous_txn_lgr_seq, Some(86724072));
        } else {
            panic!("Expected ModifiedNode");
        }

        // Test second affected node (DeletedNode)
        let second_node = &metadata.affected_nodes[1];
        if let Some(deleted) = &second_node.deleted_node {
            assert_eq!(deleted.ledger_entry_type, "Offer");
            assert_eq!(
                deleted.ledger_index,
                "348AF66EBD872FBF2BD23085D3FB4A200E15509451475027C4A5EE8D8B77C623"
            );
        } else {
            panic!("Expected DeletedNode");
        }
    }

    #[test]
    fn test_affected_node_variants() {
        let created_json = json!({
            "CreatedNode": {
                "LedgerEntryType": "AccountRoot",
                "LedgerIndex": "ABCD",
                "NewFields": {
                    "Account": "rXXX",
                    "Balance": "1000000"
                }
            }
        });

        let modified_json = json!({
            "ModifiedNode": {
                "LedgerEntryType": "AccountRoot",
                "LedgerIndex": "DEFG",
                "FinalFields": {
                    "Account": "rYYY",
                    "Balance": "2000000"
                },
                "PreviousFields": {
                    "Balance": "1000000"
                }
            }
        });

        let deleted_json = json!({
            "DeletedNode": {
                "LedgerEntryType": "Offer",
                "LedgerIndex": "HIJK",
                "FinalFields": {
                    "Account": "rZZZ"
                }
            }
        });

        let created: AffectedNode = serde_json::from_value(created_json).unwrap();
        let modified: AffectedNode = serde_json::from_value(modified_json).unwrap();
        let deleted: AffectedNode = serde_json::from_value(deleted_json).unwrap();

        assert!(created.created_node.is_some());
        assert!(created.modified_node.is_none());
        assert!(created.deleted_node.is_none());

        assert!(modified.created_node.is_none());
        assert!(modified.modified_node.is_some());
        assert!(modified.deleted_node.is_none());

        assert!(deleted.created_node.is_none());
        assert!(deleted.modified_node.is_none());
        assert!(deleted.deleted_node.is_some());
    }
}
