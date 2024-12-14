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
    /// Text result code indicating the preliminary result of the transaction, for example `tesSUCCESS`.
    pub engine_result: Cow<'a, str>,
    /// Numeric version of the result code. **Not recommended**.
    pub engine_result_code: i32,
    /// Human-readable explanation of the transaction's preliminary result.
    pub engine_result_message: Cow<'a, str>,
    /// The complete transaction in hex string format.
    pub tx_blob: Cow<'a, str>,
    /// The complete transaction in JSON format.
    pub tx_json: Value,
    /// (Omitted in sign-and-submit mode) The value true indicates that the transaction was applied,
    /// queued, broadcast, or kept for later. The value false indicates that none of those happened,
    /// so the transaction cannot possibly succeed as long as you do not submit it again and have not
    /// already submitted it another time.
    pub accepted: Option<bool>,
    /// (Omitted in sign-and-submit mode) The next Sequence Number available for the sending account after
    /// all pending and queued transactions.
    pub account_sequence_available: Option<u32>,
    /// (Omitted in sign-and-submit mode) The next Sequence Number for the sending account after all
    /// transactions that have been provisionally applied, but not transactions in the queue.
    pub account_sequence_next: Option<u32>,
    /// (Omitted in sign-and-submit mode) The value true indicates that this transaction was applied to
    /// the open ledger. In this case, the transaction is likely, but not guaranteed, to be validated in
    /// the next ledger version.
    pub applied: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates this transaction was broadcast to peer
    /// servers in the peer-to-peer XRP Ledger network. (Note: if the server has no peers, such as in
    /// stand-alone mode, the server uses the value true for cases where it would have broadcast the
    /// transaction.) The value false indicates the transaction was not broadcast to any other servers.
    pub broadcast: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates that the transaction was kept to
    /// be retried later.
    pub kept: Option<bool>,
    /// (Omitted in sign-and-submit mode) The value true indicates the transaction was put in the Transaction
    /// Queue, which means it is likely to be included in a future ledger version.
    pub queued: Option<bool>,
    /// (Omitted in sign-and-submit mode) The current open ledger cost before processing this transaction.
    /// Transactions with a lower cost are likely to be queued.
    pub open_ledger_cost: Option<Cow<'a, str>>,
    /// (Omitted in sign-and-submit mode) The ledger index of the newest validated ledger at the time
    /// of submission. This provides a lower bound on the ledger versions that the transaction can appear
    /// in as a result of this request. (The transaction could only have been validated in this ledger
    /// version or earlier if it had already been submitted before.)
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
