use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};

use crate::models::{
    amount::XRPAmount, results::exceptions::XRPLResultException, XRPLModelException,
    XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fee<'a> {
    pub current_ledger_size: Cow<'a, str>,
    pub current_queue_size: Cow<'a, str>,
    pub drops: Drops<'a>,
    pub expected_ledger_size: Cow<'a, str>,
    pub ledger_current_index: u32,
    pub levels: Levels<'a>,
    pub max_queue_size: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Levels<'a> {
    pub median_level: Cow<'a, str>,
    pub minimum_level: Cow<'a, str>,
    pub open_ledger_level: Cow<'a, str>,
    pub reference_level: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Drops<'a> {
    pub base_fee: XRPAmount<'a>,
    pub median_fee: XRPAmount<'a>,
    pub minimum_fee: XRPAmount<'a>,
    pub open_ledger_fee: XRPAmount<'a>,
}

impl<'a> TryFrom<XRPLResult<'a>> for Fee<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Fee(fee) => Ok(fee),
            res => Err(XRPLResultException::UnexpectedResultType(
                "Fee".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
