use core::convert::TryFrom;

use alloc::borrow::Cow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{models::results::exceptions::XRPLResultException, Err};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Submit<'a> {
    engine_result: Cow<'a, str>,
    engine_result_code: i32,
    engine_result_message: Cow<'a, str>,
    tx_blob: Cow<'a, str>,
    tx_json: Value,
    accepted: Option<bool>,
    account_sequence_available: Option<u32>,
    account_sequence_next: Option<u32>,
    applied: Option<bool>,
    broadcast: Option<bool>,
    kept: Option<bool>,
    queued: Option<bool>,
    open_ledger_cost: Option<Cow<'a, str>>,
    validated_ledger_index: Option<u32>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Submit<'a> {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult<'a>) -> Result<Self> {
        match result {
            XRPLResult::Submit(server_state) => Ok(server_state),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "Submit".to_string(),
                res.get_name()
            )),
        }
    }
}
