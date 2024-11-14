use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};

use crate::models::{
    amount::XRPAmount, results::exceptions::XRPLResultException, XRPLModelException,
    XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerState<'a> {
    pub state: State<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State<'a> {
    pub build_version: Cow<'a, str>,
    pub network_id: Option<u32>,
    pub validated_ledger: Option<ValidatedLedger<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatedLedger<'a> {
    pub base_fee: XRPAmount<'a>,
    pub close_time: u32,
    pub hash: Cow<'a, str>,
    pub reserve_base: XRPAmount<'a>,
    pub reserve_inc: XRPAmount<'a>,
    pub seq: u32,
}

impl<'a> TryFrom<XRPLResult<'a>> for ServerState<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::ServerState(server_state) => Ok(server_state),
            res => Err(XRPLResultException::UnexpectedResultType(
                "ServerState".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
