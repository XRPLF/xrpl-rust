use core::convert::TryFrom;

use alloc::string::ToString;
use serde::{Deserialize, Serialize};

use crate::models::{ledger::objects::AccountRoot, XRPLModelException, XRPLModelResult};

use super::{exceptions::XRPLResultException, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo<'a> {
    pub account_data: AccountRoot<'a>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::AccountInfo(account_info) => Ok(account_info),
            res => Err(XRPLResultException::UnexpectedResultType(
                "AccountInfo".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
