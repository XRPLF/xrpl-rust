//! General XRPL Binary Codec Exceptions.

use crate::utils::exceptions::ISOCodeException;
use crate::utils::exceptions::XRPRangeException;

#[derive(Debug, Clone)]
pub enum XRPLBinaryCodecException {
    UnexpectedParserSkipOverflow { max: usize, found: usize },
    UnexpectedLengthPrefixRange { min: usize, max: usize },
    UnexpectedTypeCodeRange { min: usize, max: usize },
    UnexpectedFieldCodeRange { min: usize, max: usize },
    UnexpectedFieldIdByteRange { min: usize, max: usize },
    UnknownFieldName,
    InvalidReadFromBytesValue,
    InvalidVariableLengthTooLarge { max: usize },
    InvalidHashLength { expected: usize, found: usize },
    InvalidPathSetFromValue,
    XRPRangeError(XRPRangeException),
    HexError(hex::FromHexError),
    SerdeJsonError(serde_json::error::Category),
    Utf8Error(alloc::string::FromUtf8Error),
    FromSliceError(core::array::TryFromSliceError),
    DecimalError(rust_decimal::Error),
    IntParseError(core::num::ParseIntError),
    ISOCodeError(ISOCodeException),
}

impl From<XRPRangeException> for XRPLBinaryCodecException {
    fn from(err: XRPRangeException) -> Self {
        XRPLBinaryCodecException::XRPRangeError(err)
    }
}

impl From<ISOCodeException> for XRPLBinaryCodecException {
    fn from(err: ISOCodeException) -> Self {
        XRPLBinaryCodecException::ISOCodeError(err)
    }
}

impl From<rust_decimal::Error> for XRPLBinaryCodecException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPLBinaryCodecException::DecimalError(err)
    }
}

impl From<hex::FromHexError> for XRPLBinaryCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLBinaryCodecException::HexError(err)
    }
}

impl From<serde_json::Error> for XRPLBinaryCodecException {
    fn from(err: serde_json::Error) -> Self {
        XRPLBinaryCodecException::SerdeJsonError(err.classify())
    }
}

impl From<core::array::TryFromSliceError> for XRPLBinaryCodecException {
    fn from(err: core::array::TryFromSliceError) -> Self {
        XRPLBinaryCodecException::FromSliceError(err)
    }
}

impl From<core::num::ParseIntError> for XRPLBinaryCodecException {
    fn from(err: core::num::ParseIntError) -> Self {
        XRPLBinaryCodecException::IntParseError(err)
    }
}

impl From<alloc::string::FromUtf8Error> for XRPLBinaryCodecException {
    fn from(err: alloc::string::FromUtf8Error) -> Self {
        XRPLBinaryCodecException::Utf8Error(err)
    }
}

impl core::fmt::Display for XRPLBinaryCodecException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLBinaryCodecException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}
