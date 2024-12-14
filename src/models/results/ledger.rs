use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    results::exceptions::XRPLResultException, XRPLModelException, XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledger<'a> {
    /// The complete ledger header data of this ledger, with some additional fields added for convenience.
    pub ledger: LedgerInner<'a>,
    /// The unique identifying hash of the entire ledger, as hexadecimal.
    pub ledger_hash: Cow<'a, str>,
    /// The Ledger Index of this ledger.
    pub ledger_index: u32,
    /// If true, this is a validated ledger version. If omitted or set to false, this ledger's data is not final.
    pub validated: Option<bool>,
    /// (Omitted unless requested with the queue parameter) Array of objects describing queued transactions,
    /// in the same order as the queue. If the request specified expand as true, members contain full
    /// representations of the transactions, in either JSON or binary depending on whether the request
    /// specified binary as true.
    pub queue_data: Option<QueueData<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerInner<'a> {
    /// Hash of all account state information in this ledger, as hexadecimal.
    pub account_hash: Cow<'a, str>,
    /// A bit-map of flags relating to the closing of this ledger.
    pub close_flags: u32,
    /// The time this ledger was closed, in seconds since the Ripple Epoch.
    pub close_time: u32,
    /// The time this ledger was closed, in human-readable format. Always uses the UTC time zone.
    pub close_time_human: Option<Cow<'a, str>>,
    /// Ledger close times are rounded to within this many seconds.
    pub close_time_resolution: u32,
    /// Whether or not this ledger has been closed.
    pub closed: bool,
    /// Unique identifying hash of the entire ledger.
    pub ledger_hash: Cow<'a, str>,
    /// The Ledger Index of this ledger, as a quoted integer.
    pub ledger_index: Cow<'a, str>,
    /// The time at which the previous ledger was closed.
    pub parent_close_time: u32,
    /// The unique identifying hash of the ledger that came immediately before this one, as hexadecimal.
    pub parent_hash: Cow<'a, str>,
    /// Total number of XRP drops in the network, as a quoted integer. (This decreases as transaction costs destroy XRP.)
    pub total_coins: Cow<'a, str>,
    /// Hash of the transaction information included in this ledger.
    pub transaction_hash: Cow<'a, str>,
    /// Transactions applied in this ledger version. By default, members are the transactions'
    /// identifying Hash strings. If the request specified expand as true, members are full representations of the transactions
    /// instead, in either JSON or binary depending on whether the request specified binary as true.
    pub transactions: Option<Vec<Cow<'a, str>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueData<'a> {
    /// The Address of the sender for this queued transaction.
    pub account: Cow<'a, str>,
    /// By default, this is a String containing the identifying hash of the transaction. If transactions are expanded in
    /// binary format, this is an object whose only field is tx_blob, containing the binary form of the transaction as a
    /// decimal string. If transactions are expanded in JSON format, this is an object containing the transaction object
    /// including the transaction's identifying hash in the hash field.
    pub tx: QueueDataTx<'a>,
    /// How many times this transaction can be retried before being dropped.
    pub retries_remaining: u32,
    /// The tentative result from preliminary transaction checking. This is always tesSUCCESS.
    pub preflight_result: Cow<'a, str>,
    /// If this transaction was left in the queue after getting a retriable (`ter`) result, this is the exact `ter` result code it got.
    pub last_result: Option<Cow<'a, str>>,
    /// Whether this transaction changes this address's ways of authorizing transactions.
    pub auth_change: Option<bool>,
    /// The Transaction Cost of this transaction, in drops of XRP.
    pub fee: Option<Cow<'a, str>>,
    /// The transaction cost of this transaction, relative to the minimum cost for this type of transaction, in fee levels.
    pub fee_level: Option<Cow<'a, str>>,
    /// The maximum amount of XRP, in drops, this transaction could potentially send or destroy.
    pub max_spend_drops: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueueDataTx<'a> {
    Hash(Cow<'a, str>),
    Json(Value),
}

impl<'a> TryFrom<XRPLResult<'a>> for Ledger<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Ledger(ledger) => Ok(ledger),
            res => Err(XRPLResultException::UnexpectedResultType(
                "Ledger".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
