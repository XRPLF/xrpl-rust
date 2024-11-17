use alloc::collections::BTreeMap;
use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::Amount;

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
    pub flags: i32,
    pub sequence: i32,
    pub account: Option<Cow<'a, str>>,
    pub low_limit: Option<BTreeMap<Cow<'a, str>, Cow<'a, str>>>,
    pub high_limit: Option<BTreeMap<Cow<'a, str>, Cow<'a, str>>>,
    pub balance: Option<Cow<'a, str>>,
    pub taker_gets: Option<Cow<'a, str>>,
    pub taker_pays: Option<Cow<'a, str>>,
    pub book_directory: Option<Cow<'a, str>>,
    pub expiration: Option<i32>,
    #[serde(rename = "NFTokens")]
    pub nftokens: Option<Vec<NFTokenMetadata<'a>>>,
    pub xchain_claim_id: Option<Cow<'a, str>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatedNodeFields<'a> {
    pub ledger_entry_type: Cow<'a, str>,
    pub ledger_index: Cow<'a, str>,
    pub new_fields: Fields<'a>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatedNode<'a> {
    pub created_node: CreatedNodeFields<'a>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModifiedNodeFields<'a> {
    pub ledger_entry_type: Cow<'a, str>,
    pub ledger_index: Cow<'a, str>,
    pub final_fields: Option<Fields<'a>>,
    pub previous_fields: Option<Fields<'a>>,
    pub previous_txn_id: Option<Cow<'a, str>>,
    pub previous_txn_lgr_seq: Option<i32>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModifiedNode<'a> {
    pub modified_node: ModifiedNodeFields<'a>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletedNodeFields<'a> {
    pub ledger_entry_type: Cow<'a, str>,
    pub ledger_index: Cow<'a, str>,
    pub final_fields: Fields<'a>,
    pub previous_fields: Option<Fields<'a>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeletedNode<'a> {
    pub deleted_node: DeletedNodeFields<'a>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionMetadata<'a> {
    pub affected_nodes: Vec<Node<'a>>,
    pub transaction_index: i32,
    pub transaction_result: Amount<'a>,
    #[serde(rename = "delivered_amount")]
    pub delivered_amount: Option<Amount<'a>>,
    // pub delivered_amount_unavailable: Option<Cow<'a, str>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Node<'a> {
    Created(CreatedNode<'a>),
    Modified(ModifiedNode<'a>),
    Deleted(DeletedNode<'a>),
}
