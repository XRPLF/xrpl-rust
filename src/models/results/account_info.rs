use core::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    models::{ledger::objects::AccountRoot, results::exceptions::XRPLResultException},
    Err,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo<'a> {
    pub account_data: AccountRoot<'a>,
}

impl<'a> TryFrom<XRPLResult<'a>> for AccountInfo<'a> {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult<'a>) -> Result<Self> {
        match result {
            XRPLResult::AccountInfo(account_info) => Ok(account_info),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "AccountInfo".to_string(),
                res.get_name()
            )),
        }
    }
}
