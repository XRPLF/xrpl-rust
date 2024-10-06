use core::convert::TryFrom;

use alloc::{borrow::Cow, vec::Vec};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{models::results::exceptions::XRPLResultException, Err};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountTx<'a> {
    pub account: Cow<'a, str>,
    pub ledger_index_min: Option<u32>,
    pub ledger_index_max: Option<u32>,
    pub limit: Option<u16>,
    pub marker: Option<Value>,
    pub transactions: Vec<Value>,
    pub validated: Option<bool>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountTx<'a> {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult<'a>) -> Result<Self> {
        match result {
            XRPLResult::AccountTx(account_tx) => Ok(account_tx),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "AccountTx".to_string(),
                res.get_name()
            )),
        }
    }
}
