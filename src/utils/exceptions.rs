//! Exception for invalid XRP Ledger amount data.

use alloc::string::String;
use thiserror_no_std::Error;

use crate::{core::exceptions::XRPLCoreException, models::XRPLModelException, XRPLSerdeJsonError};

pub type XRPLUtilsResult<T, E = XRPLUtilsException> = core::result::Result<T, E>;

#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLUtilsException {
    #[error("XRPL Time Range error: {0}")]
    XRPLTimeRangeError(#[from] XRPLTimeRangeException),
    #[error("XRP Range error: {0}")]
    XRPRangeError(#[from] XRPRangeException),
    #[error("XRPL NFT ID error: {0}")]
    XRPLNFTIdError(#[from] XRPLNFTIdException),
    #[error("XRPL Core error: {0}")]
    XRPLCoreError(#[from] XRPLCoreException),
    #[error("XRPL Model error: {0}")]
    XRPLModelError(#[from] XRPLModelException),
    #[error("XRPL XChain Claim ID error: {0}")]
    XRPLXChainClaimIdError(#[from] XRPLXChainClaimIdException),
    #[error("ISO Code error: {0}")]
    ISOCodeError(#[from] ISOCodeException),
    #[error("Decimal error: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("BigDecimal error: {0}")]
    BigDecimalError(#[from] bigdecimal::ParseBigDecimalError),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("From Hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
    #[error("ParseInt error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLTimeRangeException {
    #[error("Invalid time before epoch (min: {min} found: {found})")]
    InvalidTimeBeforeEpoch { min: i64, found: i64 },
    #[error("Invalid time after epoch (max: {max} found: {found})")]
    UnexpectedTimeOverflow { max: i64, found: i64 },
    #[error("Invalid local time")]
    InvalidLocalTime,
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPRangeException {
    #[error("Invalid XRP amount")]
    InvalidXRPAmount,
    #[error("Invalid Issued Currency amount")]
    InvalidICAmount,
    #[error("Invalid value contains decimal")]
    InvalidValueContainsDecimal,
    #[error("Invalid XRP amount too small (min: {min} found: {found})")]
    InvalidXRPAmountTooSmall { min: String, found: String },
    #[error("Invalid XRP amount too large (max: {max} found: {found})")]
    InvalidXRPAmountTooLarge { max: u64, found: String },
    #[error("Invalid Issued Currency precision too small (min: {min} found: {found})")]
    InvalidICPrecisionTooSmall { min: i32, found: i32 },
    #[error("Invalid Issued Currency precision too large (max: {max} found: {found})")]
    InvalidICPrecisionTooLarge { max: i32, found: i32 },
    #[error("Invalid Drops amount too large (max: {max} found: {found})")]
    InvalidDropsAmountTooLarge { max: String, found: String },
    #[error("Invalid Issued Currency serialization length (expected: {expected} found: {found})")]
    InvalidICSerializationLength { expected: usize, found: usize },
    #[error("Invalid Issued Currency amount overflow (max: {max} found: {found})")]
    UnexpectedICAmountOverflow { max: usize, found: usize },
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLNFTIdException {
    #[error("Invalid NFT ID length (expected: {expected} found: {found})")]
    InvalidNFTIdLength { expected: usize, found: usize },
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLXChainClaimIdException {
    #[error("No XChainOwnedClaimID created")]
    NoXChainOwnedClaimID,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLXChainClaimIdException {}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum ISOCodeException {
    #[error("Invalid ISO code")]
    InvalidISOCode,
    #[error("Invalid ISO length")]
    InvalidISOLength,
    #[error("Invalid XRP bytes")]
    InvalidXRPBytes,
    #[error("Invalid Currency representation")]
    UnsupportedCurrencyRepresentation,
    #[error("Invalid UTF-8")]
    Utf8Error,
}

impl From<core::str::Utf8Error> for ISOCodeException {
    fn from(_: core::str::Utf8Error) -> Self {
        ISOCodeException::Utf8Error
    }
}

impl From<serde_json::Error> for XRPLUtilsException {
    fn from(error: serde_json::Error) -> Self {
        XRPLUtilsException::SerdeJsonError(error.into())
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLTimeRangeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPRangeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for ISOCodeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLUtilsException {}
