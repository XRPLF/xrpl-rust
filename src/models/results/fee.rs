use core::convert::TryFrom;

use alloc::string::ToString;
use serde::{Deserialize, Serialize};

use crate::models::{
    amount::XRPAmount, results::exceptions::XRPLResultException, XRPLModelException,
    XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fee<'a> {
    pub drops: Drops<'a>,
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
