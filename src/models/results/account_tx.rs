use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    results::exceptions::XRPLResultException, XRPLModelException, XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountTx<'a> {
    /// Unique Address identifying the related account
    pub account: Cow<'a, str>,
    /// The ledger index of the earliest ledger actually searched for transactions.
    pub ledger_index_min: Option<u32>,
    /// The ledger index of the most recent ledger actually searched for transactions.
    pub ledger_index_max: Option<u32>,
    /// The limit value used in the request. (This may differ from the actual limit value enforced by the server.)
    pub limit: Option<u16>,
    /// Server-defined value indicating the response is paginated. Pass this to the next call to resume where this call left off.
    pub marker: Option<Value>,
    /// (JSON mode) The transaction results metadata in JSON.
    pub meta: Option<Value>, // TODO: replace with transaction metadata as soon as implemented
    /// (Binary mode) The transaction results metadata as a hex string.
    pub meta_blob: Option<Cow<'a, str>>,
    /// Array of transactions matching the request's criteria, as explained below.
    pub transactions: Vec<Value>,
    /// If included and set to true, the information in this response comes from a validated ledger version. Otherwise,
    /// the information is subject to change.
    pub validated: Option<bool>,
}

pub struct Transaction<'a> {
    /// The ledger close time represented in ISO 8601 time format.
    pub close_time_iso: Cow<'a, str>,
    /// The unique hash identifier of the transaction.
    pub hash: Cow<'a, str>,
    /// A hex string of the ledger version that included this transaction.
    pub ledger_hash: Cow<'a, str>,
    /// The ledger index of the ledger version that included this transaction.
    pub ledger_index: u32,
    /// Whether or not the transaction is included in a validated ledger. Any transaction not yet in a validated ledger is subject to change.
    pub validated: bool,
    /// (JSON mode) JSON object defining the transaction.
    pub tx_json: Option<Value>,
    /// (Binary mode) A unique hex string defining the transaction.
    pub tx_blob: Option<Cow<'a, str>>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountTx<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::AccountTx(account_tx) => Ok(account_tx),
            res => Err(XRPLResultException::UnexpectedResultType(
                "AccountTx".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
