use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::requests::Marker;

use super::{exceptions::XRPLResultException, XRPLResponse, XRPLResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTHistory<'a> {
    pub nft_id: Cow<'a, str>,
    pub ledger_index_min: u32,
    pub ledger_index_max: u32,
    pub transactions: Vec<NFTHistoryTransaction<'a>>,
    pub limit: Option<u32>,
    pub marker: Option<Marker<'a>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NFTHistoryTransaction<'a> {
    pub hash: Cow<'a, str>,
    pub ledger_index: u32,
    pub meta: NFTHistoryTransactionMeta<'a>,
    pub validated: bool,
    pub tx: Option<Value>,
    pub tx_blob: Option<Cow<'a, str>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum NFTHistoryTransactionMeta<'a> {
    Json(Value),
    Blob(Cow<'a, str>),
}

impl<'a> TryFrom<XRPLResult<'a>> for NFTHistory<'a> {
    type Error = crate::models::XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> crate::models::XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTHistory(nft_history) => Ok(nft_history),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTHistory".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl<'a> TryFrom<XRPLResponse<'a>> for NFTHistory<'a> {
    type Error = crate::models::XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> crate::models::XRPLModelResult<Self> {
        match response.result {
            Some(result) => NFTHistory::try_from(result),
            None => Err(crate::models::XRPLModelException::MissingField(
                "result".to_string(),
            )),
        }
    }
}
