use core::convert::TryFrom;

use alloc::{borrow::Cow, vec::Vec};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{models::results::exceptions::XRPLResultException, Err};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledger<'a> {
    pub ledger: LedgerInner<'a>,
    pub ledger_hash: Cow<'a, str>,
    pub ledger_index: u32,
    pub validated: Option<bool>,
    pub queue_data: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerInner<'a> {
    pub account_hash: Cow<'a, str>,
    pub close_flags: u32,
    pub close_time: u32,
    pub close_time_human: Option<Cow<'a, str>>,
    pub close_time_resolution: u32,
    pub closed: bool,
    pub ledger_hash: Cow<'a, str>,
    pub ledger_index: Cow<'a, str>,
    pub parent_close_time: u32,
    pub parent_hash: Cow<'a, str>,
    pub total_coins: Cow<'a, str>,
    pub transaction_hash: Cow<'a, str>,
    pub transactions: Option<Vec<Cow<'a, str>>>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Ledger<'a> {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult<'a>) -> Result<Self> {
        match result {
            XRPLResult::Ledger(ledger) => Ok(ledger),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "ledger-models".to_string(),
                res.get_name()
            )),
        }
    }
}
