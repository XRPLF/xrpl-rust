use core::num::ParseIntError;

use alloc::borrow::Cow;
use thiserror_no_std::Error;

use crate::models::amount::XRPAmount;

#[derive(Error, Debug, PartialEq)]
pub enum XRPLTransactionException<'a> {
    #[error("Fee of {0:?} Drops is much higher than a typical XRP transaction fee. This may be a mistake. If intentional, please use `check_fee = false`")]
    FeeUnusuallyHigh(XRPAmount<'a>),
    #[error("Unable to parse rippled version: {0}")]
    ParseRippledVersionError(ParseIntError),
    #[error("Invalid rippled version: {0}")]
    InvalidRippledVersion(Cow<'a, str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLSignTransactionException<'a> {
    #[error("{0:?} value does not match X-Address tag")]
    TagFieldMismatch(&'a str),
    #[error("Fee value of {0:?} is likely entered incorrectly, since it is much larger than the typical XRP transaction cost. If this is intentional, use `check_fee=Some(false)`.")]
    FeeTooHigh(Cow<'a, str>),
    #[error("Wallet is required to sign transaction")]
    WalletRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLSubmitAndWaitException<'a> {
    #[error("Transaction submission failed: {0}")]
    SubmissionFailed(Cow<'a, str>),
    #[error("The latest validated ledger sequence {validated_ledger_sequence} is greater than the LastLedgerSequence {last_ledger_sequence} in the Transaction. Prelim result: {prelim_result}")]
    SubmissionTimeout {
        last_ledger_sequence: u32,
        validated_ledger_sequence: u32,
        prelim_result: Cow<'a, str>,
    },
    #[error("Expected field in the transaction metadata: {0}")]
    ExpectedFieldInTxMeta(Cow<'a, str>),
}
