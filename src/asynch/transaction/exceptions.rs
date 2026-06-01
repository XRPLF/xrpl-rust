use core::num::ParseIntError;

use alloc::string::String;
use thiserror_no_std::Error;

// XRPLSignTransactionException now lives in `crate::signing::exceptions`.
// Re-exported here for backward compatibility.
pub use crate::signing::exceptions::XRPLSignTransactionException;

#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum XRPLTransactionHelperException {
    #[error("Fee of {0:?} Drops is much higher than a typical XRP transaction fee. This may be a mistake. If intentional, please use `check_fee = false`")]
    FeeUnusuallyHigh(String),
    #[error("Unable to parse rippled version: {0}")]
    ParseRippledVersionError(ParseIntError),
    #[error("Invalid rippled version: {0}")]
    InvalidRippledVersion(String),
    #[error("XRPL Sign Transaction error: {0}")]
    XRPLSignTransactionError(#[from] XRPLSignTransactionException),
    #[error("XRPL Submit and Wait error: {0}")]
    XRPLSubmitAndWaitError(#[from] XRPLSubmitAndWaitException),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLSubmitAndWaitException {
    #[error("Transaction submission failed: {0}")]
    SubmissionFailed(String),
    #[error("The latest validated ledger sequence {validated_ledger_sequence} is greater than the LastLedgerSequence {last_ledger_sequence} in the Transaction. Prelim result: {prelim_result}")]
    SubmissionTimeout {
        last_ledger_sequence: u32,
        validated_ledger_sequence: u32,
        prelim_result: String,
    },
    #[error("Expected field in the transaction metadata: {0}")]
    ExpectedFieldInTxMeta(String),
}
