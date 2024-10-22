//! Exception for invalid XRP Ledger type data.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::exceptions::ISOCodeException;
use crate::utils::exceptions::JSONParseException;
use crate::utils::exceptions::XRPRangeException;
use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLTypeException {
    #[error("Invalid None value")]
    InvalidNoneValue,
    #[error("From hex error")]
    FromHexError,
    #[error("XRPL Binary Codec error: {0}")]
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    #[error("XRPL Hash error: {0}")]
    XRPLHashError(XRPLHashException),
    #[error("XRPL Range error: {0}")]
    XRPLRangeError(XRPRangeException),
    #[error("XRPL XChain Bridge error: {0}")]
    XRPLXChainBridgeError(XRPLXChainBridgeException),
    #[error("Decimal error: {0}")]
    DecimalError(rust_decimal::Error),
    #[error("Big Decimal error: {0}")]
    BigDecimalError(bigdecimal::ParseBigDecimalError),
    #[error("JSON Parse error: {0}")]
    JSONParseError(JSONParseException),
    #[error("Unknown XRPL type")]
    UnknownXRPLType,
    #[error("Missing field: {0}")]
    MissingField(&'static str),
    #[error("Unexpected JSON type")]
    UnexpectedJSONType,
    #[error("XRPL Address codec error: {0}")]
    XRPLAddressCodecException(#[from] XRPLAddressCodecException),
    #[error("XRPL Serialize Map error: {0}")]
    XRPLSerializeMapException(#[from] XRPLSerializeMapException),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum XRPLSerializeArrayException {
    #[error("Expected `Value` to be an array.")]
    ExpectedArray,
    #[error("Expected `Value` to be an array of objects.")]
    ExpectedObjectArray,
}

#[derive(Debug, Clone, PartialEq, Error)]
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
#[non_exhaustive]
pub enum XRPLHashException {
    #[error("Invalid hash length (expected {expected}, found {found})")]
    InvalidHashLength { expected: usize, found: usize },
    #[error("Invalid hash length")]
    FromHexError,
    #[error("ISO code error: {0}")]
    ISOCodeError(ISOCodeException),
    #[error("XRPL Binary Codec error: {0}")]
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    #[error("XRPL Address Codec error: {0}")]
    XRPLAddressCodecError(XRPLAddressCodecException),
    #[error("Serde JSON error")]
    SerdeJsonError(serde_json::error::Category),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLVectorException {
    #[error("Invalid vector 256 bytes")]
    InvalidVector256Bytes,
    #[error("XRPL Binary Codec error: {0}")]
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    #[error("XRPL Hash error: {0}")]
    XRPLHashError(XRPLHashException),
}

impl From<XRPLHashException> for XRPLTypeException {
    fn from(err: XRPLHashException) -> Self {
        XRPLTypeException::XRPLHashError(err)
    }
}

impl From<XRPRangeException> for XRPLTypeException {
    fn from(err: XRPRangeException) -> Self {
        XRPLTypeException::XRPLRangeError(err)
    }
}

impl From<XRPLBinaryCodecException> for XRPLTypeException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        XRPLTypeException::XRPLBinaryCodecError(err)
    }
}

impl From<JSONParseException> for XRPLTypeException {
    fn from(err: JSONParseException) -> Self {
        XRPLTypeException::JSONParseError(err)
    }
}

impl From<rust_decimal::Error> for XRPLTypeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPLTypeException::DecimalError(err)
    }
}

impl From<hex::FromHexError> for XRPLTypeException {
    fn from(_: hex::FromHexError) -> Self {
        XRPLTypeException::FromHexError
    }
}

impl From<bigdecimal::ParseBigDecimalError> for XRPLTypeException {
    fn from(err: bigdecimal::ParseBigDecimalError) -> Self {
        XRPLTypeException::BigDecimalError(err)
    }
}

impl From<XRPLXChainBridgeException> for XRPLTypeException {
    fn from(err: XRPLXChainBridgeException) -> Self {
        XRPLTypeException::XRPLXChainBridgeError(err)
    }
}

impl From<ISOCodeException> for XRPLHashException {
    fn from(err: ISOCodeException) -> Self {
        XRPLHashException::ISOCodeError(err)
    }
}

impl From<XRPLBinaryCodecException> for XRPLHashException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        XRPLHashException::XRPLBinaryCodecError(err)
    }
}

impl From<XRPLAddressCodecException> for XRPLHashException {
    fn from(err: XRPLAddressCodecException) -> Self {
        XRPLHashException::XRPLAddressCodecError(err)
    }
}

impl From<serde_json::Error> for XRPLHashException {
    fn from(err: serde_json::Error) -> Self {
        XRPLHashException::SerdeJsonError(err.classify())
    }
}

impl From<hex::FromHexError> for XRPLHashException {
    fn from(_: hex::FromHexError) -> Self {
        XRPLHashException::FromHexError
    }
}

impl From<XRPLHashException> for XRPLVectorException {
    fn from(err: XRPLHashException) -> Self {
        XRPLVectorException::XRPLHashError(err)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLTypeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLHashException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLVectorException {}
