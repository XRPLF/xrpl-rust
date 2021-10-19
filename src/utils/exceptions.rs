//! Exception for invalid XRP Ledger amount data.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::string::String;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPRangeException {
    InvalidXRPAmountTooSmall { min: String, found: String },
    InvalidXRPAmountTooLarge { max: u64, found: u64 },
    InvalidDropsAmountTooLarge { max: String, found: String },
    DecimalError(rust_decimal::Error),
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

#[cfg(feature = "std")]
impl alloc::error::Error for XRPRangeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for ISOCodeException {}

#[cfg(feature = "std")]
impl alloc::fmt::Display for XRPRangeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPRangeException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::fmt::Display for ISOCodeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "ISOCodeException: {:?}", self)
    }
}
