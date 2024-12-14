use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    results::exceptions::XRPLResultException, XRPLModelException, XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx<'a> {
    pub ctid: Cow<'a, str>,
    pub date: u32,
    pub hash: Cow<'a, str>,
    pub ledger_index: u32,
    pub meta: Value, // TODO: replace with metadata as soon as implemented
    pub meta_blob: Cow<'a, str>,
    pub tx_blob: Cow<'a, str>,
    pub tx_json: Value,
    pub validated: Option<bool>,
    /// (Deprecated) Alias for `ledger_index`
    #[serde(rename = "inLedger")]
    pub in_ledger: Option<u32>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Tx<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Tx(tx) => Ok(tx),
            res => Err(
                XRPLResultException::UnexpectedResultType("Tx".to_string(), res.get_name()).into(),
            ),
        }
    }
}
