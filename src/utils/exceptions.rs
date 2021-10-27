//! Exception for invalid XRP Ledger amount data.

use alloc::string::String;

#[derive(Debug, Clone)]
pub enum XRPLTimeRangeException {
    InvalidTimeBeforeEpoch { min: i64, found: i64 },
    UnexpectedTimeOverflow { max: i64, found: i64 },
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPRangeException {
    InvalidXRPAmount,
    InvalidICAmount,
    InvalidValueContainsDecimal,
    InvalidXRPAmountTooSmall { min: String, found: String },
    InvalidXRPAmountTooLarge { max: u64, found: String },
    InvalidICPrecisionTooSmall { min: i32, found: i32 },
    InvalidICPrecisionTooLarge { max: i32, found: i32 },
    InvalidDropsAmountTooLarge { max: String, found: String },
    InvalidICSerializationLength { expected: usize, found: usize },
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
    InvalidSerdeValue {
        expected: String,
        found: serde_json::Value,
    },
    UnsupportedCurrencyRepresentation,
    DecimalError(rust_decimal::Error),
    HexError(hex::FromHexError),
    Utf8Error(core::str::Utf8Error),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum JSONParseException {
    ISOCodeError(ISOCodeException),
    DecimalError(rust_decimal::Error),
    XRPRangeError(XRPRangeException),
    InvalidSerdeValue {
        expected: String,
        found: serde_json::Value,
    },
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

impl From<rust_decimal::Error> for ISOCodeException {
    fn from(err: rust_decimal::Error) -> Self {
        ISOCodeException::DecimalError(err)
    }
}

impl From<core::str::Utf8Error> for ISOCodeException {
    fn from(err: core::str::Utf8Error) -> Self {
        ISOCodeException::Utf8Error(err)
    }
}

impl From<hex::FromHexError> for ISOCodeException {
    fn from(err: hex::FromHexError) -> Self {
        ISOCodeException::HexError(err)
    }
}

impl From<XRPRangeException> for JSONParseException {
    fn from(err: XRPRangeException) -> Self {
        JSONParseException::XRPRangeError(err)
    }
}

impl From<ISOCodeException> for JSONParseException {
    fn from(err: ISOCodeException) -> Self {
        JSONParseException::ISOCodeError(err)
    }
}

impl From<rust_decimal::Error> for JSONParseException {
    fn from(err: rust_decimal::Error) -> Self {
        JSONParseException::DecimalError(err)
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

impl core::fmt::Display for XRPLTimeRangeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPLTimeRangeException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPRangeException {}

#[cfg(feature = "std")]
impl alloc::error::Error for ISOCodeException {}
