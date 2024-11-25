use alloc::collections::BTreeMap;
use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::ledger::objects::LedgerEntryType;
use crate::models::requests::LedgerIndex;
use crate::models::{Amount, IssuedCurrencyAmount};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMetadata<'a> {
    #[serde(rename = "NFToken")]
    pub nftoken: NFTokenMetadataFields<'a>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMetadataFields<'a> {
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: Cow<'a, str>,
    #[serde(rename = "URI")]
    pub uri: Cow<'a, str>,
}
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fields<'a> {
    pub account: Option<Cow<'a, str>>,
    pub balance: Option<Amount<'a>>,
    pub book_directory: Option<Cow<'a, str>>,
    pub expiration: Option<u32>,
    pub flags: u32,
    pub low_limit: Option<IssuedCurrencyAmount<'a>>,
    pub high_limit: Option<IssuedCurrencyAmount<'a>>,
    pub next_page_min: Option<Cow<'a, str>>,
    #[serde(rename = "NFTokens")]
    pub nftokens: Option<Vec<NFTokenMetadata<'a>>>,
    pub previous_page_min: Option<Cow<'a, str>>,
    pub sequence: u32,
    pub taker_gets: Option<Amount<'a>>,
    pub taker_pays: Option<Amount<'a>>,
    pub xchain_claim_id: Option<Cow<'a, str>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AffectedNode<'a> {
    #[serde(rename_all = "PascalCase")]
    CreatedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        new_fields: Fields<'a>,
    },
    #[serde(rename_all = "PascalCase")]
    ModifiedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        final_fields: Option<Fields<'a>>,
        previous_fields: Option<Fields<'a>>,
        previous_txn_id: Option<Cow<'a, str>>,
        previous_txn_lgr_seq: Option<u32>,
    },
    #[serde(rename_all = "PascalCase")]
    DeletedNode {
        ledger_entry_type: LedgerEntryType,
        ledger_index: LedgerIndex<'a>,
        final_fields: Fields<'a>,
        previous_fields: Option<Fields<'a>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    CreatedNode,
    ModifiedNode,
    DeletedNode,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMetadata<'a> {
    pub affected_nodes: Vec<AffectedNode<'a>>,
    pub transaction_index: u32,
    pub transaction_result: Amount<'a>,
    #[serde(rename = "delivered_amount")]
    pub delivered_amount: Option<Amount<'a>>,
}

#[cfg(test)]
mod test_serde {
    #[test]
    fn test_deserialize_deleted_node() {
        let json = r#"
            {
                "DeletedNode": {
                    "FinalFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "BookDirectory": "623C4C4AD65873DA787AC85A0A1385FE6233B6DE100799474F20E441AE211B08",
                        "BookNode": "0",
                        "Flags": 0,
                        "OwnerNode": "0",
                        "PreviousTxnID": "3C5524789C20AE661CF4985EF694F11128FE267D084CB5E77DBB5FFD78E31E1C",
                        "PreviousTxnLgrSeq": 92173580,
                        "Sequence": 29337056,
                        "TakerGets": "17250658754",
                        "TakerPays": {
                            "currency": "CNY",
                            "issuer": "rJ1adrpGS3xsnQMb9Cw54tWJVFPuSdZHK",
                            "value": "159709.5313591656"
                        }
                    },
                    "LedgerEntryType": "Offer",
                    "LedgerIndex": "D11F69DE8A8CACB130F2E2B9893E5C97B9EE4136759C66C1F3497C8575FF5ED0"
                }
            }
        "#;
        let deleted_node = serde_json::from_str::<super::AffectedNode>(json);

        assert!(deleted_node.is_ok());
    }

    #[test]
    fn test_deserialize_modified_node() {
        let json = r#"
            {
                "ModifiedNode": {
                    "FinalFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "Balance": "5000542889",
                        "Flags": 0,
                        "OwnerCount": 5,
                        "Sequence": 29337064
                    },
                    "LedgerEntryType": "AccountRoot",
                    "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                    "PreviousFields": {
                        "Balance": "5000542904",
                        "Sequence": 29337063
                    },
                    "PreviousTxnID": "960FAFAF9CA0465B7475F888946F0D58F9CF49B18F3991D826B03A5025368DDE",
                    "PreviousTxnLgrSeq": 92173588
                }
            }
        "#;
        let modified_node = serde_json::from_str::<super::AffectedNode>(json);

        assert!(modified_node.is_ok());
    }

    #[test]
    fn test_deserialize_created_node() {
        let json = r#"
            {
                "CreatedNode": {
                    "LedgerEntryType": "AccountRoot",
                    "LedgerIndex": "991ED60C316200D33B2EA3E56E505433394DBA7FF5E7ADE8C8850D02BEF1F53A",
                    "NewFields": {
                        "Account": "rHzKtpcB1KC1YuU4PBhk9m2abqrf2kZsfV",
                        "Balance": "5000542889",
                        "Flags": 0,
                        "OwnerCount": 5,
                        "Sequence": 29337064
                    }
                }
            }
        "#;
        let created_node = serde_json::from_str::<super::AffectedNode>(json);

        assert!(created_node.is_ok());
    }
}
