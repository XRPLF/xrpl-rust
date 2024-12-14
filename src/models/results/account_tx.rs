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
    pub account: Cow<'a, str>,
    pub ledger_index_min: Option<u32>,
    pub ledger_index_max: Option<u32>,
    pub limit: Option<u16>,
    pub marker: Option<Value>,
    pub meta: Option<Value>,
    pub meta_blob: Option<Cow<'a, str>>,
    pub transactions: Vec<Value>, // TODO: replace with transaction metadata as soon as implemented
    pub validated: Option<bool>,
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
