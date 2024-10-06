//! General XRPL Binary Codec Exceptions.

use crate::utils::exceptions::ISOCodeException;
use crate::utils::exceptions::XRPRangeException;
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
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
    TryFromSliceError,
    TryFromIntError,
    FromUtf8Error,
    ParseIntError,
    FromHexError,
    XRPRangeError(XRPRangeException),
    SerdeJsonError(serde_json::error::Category),
    DecimalError(rust_decimal::Error),
    ISOCodeError(ISOCodeException),
    FieldHasNoAssiciatedTag,
    XAddressTagMismatch,
    FieldIsNotAccountOrDestination,
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
    fn from(_: hex::FromHexError) -> Self {
        XRPLBinaryCodecException::FromHexError
    }
}

impl From<serde_json::Error> for XRPLBinaryCodecException {
    fn from(err: serde_json::Error) -> Self {
        XRPLBinaryCodecException::SerdeJsonError(err.classify())
    }
}

impl From<core::num::TryFromIntError> for XRPLBinaryCodecException {
    fn from(_: core::num::TryFromIntError) -> Self {
        XRPLBinaryCodecException::TryFromIntError
    }
}

impl From<core::array::TryFromSliceError> for XRPLBinaryCodecException {
    fn from(_: core::array::TryFromSliceError) -> Self {
        XRPLBinaryCodecException::TryFromSliceError
    }
}

impl From<core::num::ParseIntError> for XRPLBinaryCodecException {
    fn from(_: core::num::ParseIntError) -> Self {
        XRPLBinaryCodecException::ParseIntError
    }
}

impl From<alloc::string::FromUtf8Error> for XRPLBinaryCodecException {
    fn from(_: alloc::string::FromUtf8Error) -> Self {
        XRPLBinaryCodecException::FromUtf8Error
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}
