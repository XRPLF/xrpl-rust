use core::convert::TryFrom;

use alloc::borrow::Cow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{models::results::exceptions::XRPLResultException, Err};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx<'a> {
    pub ctid: Cow<'a, str>,
    pub date: u32,
    pub hash: Cow<'a, str>,
    pub ledger_index: u32,
    pub meta: Value,
    /// Various fields of the transaction
    #[serde(flatten)]
    pub various: Value,
    pub validated: Option<bool>,
    /// (Deprecated) Alias for `ledger_index`
    #[serde(rename = "inLedger")]
    pub in_ledger: Option<u32>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Tx<'a> {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult<'a>) -> Result<Self> {
        match result {
            XRPLResult::Tx(tx) => Ok(tx),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "Tx".to_string(),
                res.get_name()
            )),
        }
    }
}
