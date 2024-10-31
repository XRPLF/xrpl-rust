//! Exception for invalid XRP Ledger type data.

use crate::utils::exceptions::XRPRangeException;
use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLTypeException {
    #[error("Invalid None value")]
    InvalidNoneValue,
    #[error("Unknown XRPL type")]
    UnknownXRPLType,
    #[error("Unexpected JSON type")]
    UnexpectedJSONType,
    #[error("Try from str error")]
    TryFromStrError,
    #[error("Failed to parse type from issued currency")]
    TryFromIssuedCurrencyError,
    #[error("XRPL Serialize Map error: {0}")]
    XRPLSerializeMapException(#[from] XRPLSerializeMapException),
    #[error("XRPL Serialize Array error: {0}")]
    XRPLSerializeArrayException(#[from] XRPLSerializeArrayException),
    #[error("XRPL Hash error: {0}")]
    XRPLHashError(#[from] XRPLHashException),
    #[error("XRPL Range error: {0}")]
    XRPLRangeError(#[from] XRPRangeException),
    #[error("XRPL XChain Bridge error: {0}")]
    XRPLXChainBridgeError(#[from] XRPLXChainBridgeException),
    #[error("XRPL Vector error: {0}")]
    XRPLVectorError(#[from] XRPLVectorException),
    #[error("Decimal error: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("Big Decimal error: {0}")]
    BigDecimalError(#[from] bigdecimal::ParseBigDecimalError),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLSerializeArrayException {
    #[error("Expected `Value` to be an array.")]
    ExpectedArray,
    #[error("Expected `Value` to be an array of objects.")]
    ExpectedObjectArray,
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLSerializeMapException {
    #[error("Expected `Value` to be an object.")]
    ExpectedObject,
    #[error("Field `{field}` is not allowed to have an associated tag.")]
    DisallowedTag { field: String },
    #[error("Cannot have mismatched Account X-Address and SourceTag")]
    AccountMismatchingTags,
    #[error("Cannot have mismatched Destination X-Address and DestinationTag")]
    DestinationMismatchingTags,
    #[error("Unknown transaction type: {0}")]
    UnknownTransactionType(String),
    #[error("Unknown transaction result: {0}")]
    UnknownTransactionResult(String),
    #[error("Unknown ledger entry type: {0}")]
    UnknownLedgerEntryType(String),
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLXChainBridgeException {
    #[error("Invalid XChainBridge type")]
    InvalidXChainBridgeType,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLHashException {
    #[error("Invalid hash length (expected {expected}, found {found})")]
    InvalidHashLength { expected: usize, found: usize },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLVectorException {
    #[error("Invalid vector 256 bytes")]
    InvalidVector256Bytes,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLTypeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLSerializeArrayException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLSerializeMapException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLXChainBridgeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLHashException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLVectorException {}
