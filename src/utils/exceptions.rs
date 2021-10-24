//! Exception for invalid XRP Ledger amount data.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::string::String;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPRangeException {
    InvalidXRPAmount,
    InvalidICAmount,
    InvalidValueContainsDecimal,
    InvalidXRPAmountTooSmall { min: String, found: String },
    InvalidXRPAmountTooLarge { max: u64, found: u64 },
    InvalidICPrecisionTooSmall { min: i32, found: i32 },
    InvalidICPrecisionTooLarge { max: i32, found: i32 },
    InvalidDropsAmountTooLarge { max: String, found: String },
    UnexpectedICAmountOverflow { max: usize, found: usize },
    DecimalError(rust_decimal::Error),
    HexError(hex::FromHexError),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ISOCodeException {
    InvalidISOCode,
    InvalidISOLength,
    InvalidXRPBytes,
    UnsupportedCurrencyRepresentation,
    HexError(hex::FromHexError),
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    Utf8Error(core::str::Utf8Error),
}

impl From<rust_decimal::Error> for XRPRangeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPRangeException::DecimalError(err)
    }
}

impl From<hex::FromHexError> for XRPRangeException {
    fn from(err: hex::FromHexError) -> Self {
        XRPRangeException::HexError(err)
    }
}

impl From<core::str::Utf8Error> for ISOCodeException {
    fn from(err: core::str::Utf8Error) -> Self {
        ISOCodeException::Utf8Error(err)
    }
}

impl From<XRPLBinaryCodecException> for ISOCodeException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        ISOCodeException::XRPLBinaryCodecError(err)
    }
}

impl From<hex::FromHexError> for ISOCodeException {
    fn from(err: hex::FromHexError) -> Self {
        ISOCodeException::HexError(err)
    }
}

impl core::fmt::Display for XRPRangeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPRangeException: {:?}", self)
    }
}

impl core::fmt::Display for ISOCodeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "ISOCodeException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPRangeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for ISOCodeException {}
