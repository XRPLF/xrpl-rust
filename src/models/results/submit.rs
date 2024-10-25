use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    results::exceptions::XRPLResultException, XRPLModelException, XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Submit<'a> {
    pub engine_result: Cow<'a, str>,
    pub engine_result_code: i32,
    pub engine_result_message: Cow<'a, str>,
    pub tx_blob: Cow<'a, str>,
    pub tx_json: Value,
    pub accepted: Option<bool>,
    pub account_sequence_available: Option<u32>,
    pub account_sequence_next: Option<u32>,
    pub applied: Option<bool>,
    pub broadcast: Option<bool>,
    pub kept: Option<bool>,
    pub queued: Option<bool>,
    pub open_ledger_cost: Option<Cow<'a, str>>,
    pub validated_ledger_index: Option<u32>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Submit<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Submit(server_state) => Ok(server_state),
            res => Err(XRPLResultException::UnexpectedResultType(
                "Submit".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
