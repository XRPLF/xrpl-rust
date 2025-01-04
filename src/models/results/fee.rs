use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString};
use serde::{Deserialize, Serialize};

use crate::models::{
    amount::XRPAmount, results::exceptions::XRPLResultException, XRPLModelException,
    XRPLModelResult,
};

use super::{XRPLResponse, XRPLResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fee<'a> {
    /// Number of transactions provisionally included in the in-progress ledger.
    pub current_ledger_size: Cow<'a, str>,
    /// Number of transactions currently queued for the next ledger.
    pub current_queue_size: Cow<'a, str>,
    /// Various information about the transaction cost (the Fee field of a transaction), in drops of XRP.
    pub drops: Drops<'a>,
    /// The approximate number of transactions expected to be included in the current ledger. This is based on the number of transactions in the previous ledger.
    pub expected_ledger_size: Cow<'a, str>,
    /// The Ledger Index of the current open ledger these stats describe.
    pub ledger_current_index: u32,
    /// Various information about the transaction cost, in fee levels. The ratio in fee levels applies to any transaction relative to the minimum cost of that particular transaction.
    pub levels: Levels<'a>,
    /// The maximum number of transactions that the transaction queue can currently hold.
    pub max_queue_size: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Levels<'a> {
    /// The median transaction cost among transactions in the previous validated ledger, represented in fee levels.
    pub median_level: Cow<'a, str>,
    /// The minimum transaction cost required to be queued for a future ledger, represented in fee levels.
    pub minimum_level: Cow<'a, str>,
    /// The minimum transaction cost required to be included in the current open ledger, represented in fee levels.
    pub open_ledger_level: Cow<'a, str>,
    /// The equivalent of the minimum transaction cost, represented in fee levels.
    pub reference_level: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Drops<'a> {
    /// The transaction cost required for a reference transaction to be included in a ledger under minimum load, represented in drops of XRP.
    pub base_fee: XRPAmount<'a>,
    /// An approximation of the median transaction cost among transactions included in the previous validated ledger, represented in drops of XRP.
    pub median_fee: XRPAmount<'a>,
    /// The minimum transaction cost for a reference transaction to be queued for a later ledger, represented in drops of XRP. If greater than base_fee, the transaction queue is full.
    pub minimum_fee: XRPAmount<'a>,
    /// The minimum transaction cost that a reference transaction must pay to be included in the current open ledger, represented in drops of XRP.
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

impl<'a> TryFrom<XRPLResponse<'a>> for Fee<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => Fee::try_from(result),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;

    const RESPONSE: &str = r#"{
        "current_ledger_size": "14",
        "current_queue_size": "0",
        "drops": {
            "base_fee": "10",
            "median_fee": "11000",
            "minimum_fee": "10",
            "open_ledger_fee": "10"
        },
        "expected_ledger_size": "24",
        "ledger_current_index": 26575101,
        "levels": {
            "median_level": "281600",
            "minimum_level": "256",
            "open_ledger_level": "256",
            "reference_level": "256"
        },
        "max_queue_size": "480"
    }"#;

    #[test]
    fn test_deserialize_fee() -> XRPLModelResult<()> {
        Ok(())
    }
}
