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
    ///	The transaction's compact transaction identifier.
    pub ctid: Cow<'a, str>,
    /// The close time of the ledger in which the transaction was applied, in seconds since the Ripple Epoch.
    pub date: u32,
    /// The unique identifying hash of the transaction
    pub hash: Cow<'a, str>,
    /// The ledger index of the ledger that includes this transaction.
    pub ledger_index: u32,
    /// (JSON mode) Transaction metadata, which describes the results of the transaction.
    pub meta: Option<Value>, // TODO: replace with metadata as soon as implemented
    /// (Binary mode) Transaction metadata, which describes the results of the transaction, represented as a hex string.
    pub meta_blob: Option<Cow<'a, str>>,
    /// (Binary mode) The transaction data represented as a hex string.
    pub tx_blob: Option<Cow<'a, str>>,
    /// The transaction data represented in JSON.
    pub tx_json: Value,
    /// If true, this data comes from a validated ledger version; if omitted or set to false, this data is not final.
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
