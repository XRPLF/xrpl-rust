use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLMultisignException {
    #[error("No signers set in the transaction. Use `sign` function with `multisign = true`.")]
    NoSigners,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLSignTransactionException {
    #[error("{0:?} value does not match X-Address tag")]
    TagFieldMismatch(String),
    #[error("Fee value of {0:?} is likely entered incorrectly, since it is much larger than the typical XRP transaction cost. If this is intentional, use `check_fee=Some(false)`.")]
    FeeTooHigh(String),
    #[error("Wallet is required to sign transaction")]
    WalletRequired,
}
