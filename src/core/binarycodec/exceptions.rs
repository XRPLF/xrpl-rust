//! General XRPL Binary Codec Exceptions.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::utils::exceptions::ISOCodeException;
use crate::utils::exceptions::XRPRangeException;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Clone, Error)]
#[non_exhaustive]
pub enum XRPLBinaryCodecException {
    #[error("Unexpected parser skip overflow: max: {max}, found: {found}")]
    UnexpectedParserSkipOverflow { max: usize, found: usize },
    #[error("Unexpected length prefix range: min: {min}, max: {max}")]
    UnexpectedLengthPrefixRange { min: usize, max: usize },
    #[error("Unexpected type code range(min: {min}, max: {max})")]
    UnexpectedTypeCodeRange { min: usize, max: usize },
    #[error("Unexpected field code range(min: {min}, max: {max})")]
    UnexpectedFieldCodeRange { min: usize, max: usize },
    #[error("Unexpected field id byte range(min: {min}, max: {max})")]
    UnexpectedFieldIdByteRange { min: usize, max: usize },
    #[error("Unknown field name")]
    UnknownFieldName,
    #[error("Invalid read from bytes value")]
    InvalidReadFromBytesValue,
    #[error("Invalid variable length too large: max: {max}")]
    InvalidVariableLengthTooLarge { max: usize },
    #[error("Invalid hash length (expected: {expected}, found: {found})")]
    InvalidHashLength { expected: usize, found: usize },
    #[error("Invalid path set from value")]
    InvalidPathSetFromValue,
    #[error("Try from slice error")]
    TryFromSliceError,
    #[error("Try from int error: {0}")]
    TryFromIntError(#[from] core::num::TryFromIntError),
    #[error("From utf8 error: {0}")]
    FromUtf8Error(#[from] alloc::string::FromUtf8Error),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),
    #[error("From hex error")]
    FromHexError(#[from] hex::FromHexError),
    #[error("XRP range error: {0}")]
    XRPRangeError(#[from] XRPRangeException),
    #[error("Serde json error")]
    SerdeJsonError,
    #[error("Decimal error: {0}")]
    DecimalError(#[from] rust_decimal::Error),
    #[error("Big decimal error: {0}")]
    BigDecimalError(#[from] bigdecimal::ParseBigDecimalError),
    #[error("ISO code error: {0}")]
    ISOCodeError(#[from] ISOCodeException),
    #[error("Field has no associated tag")]
    FieldHasNoAssiciatedTag,
    #[error("XAddress tag mismatch")]
    XAddressTagMismatch,
    #[error("Field is not account or destination")]
    FieldIsNotAccountOrDestination,
}

impl From<core::array::TryFromSliceError> for XRPLBinaryCodecException {
    fn from(_: core::array::TryFromSliceError) -> Self {
        XRPLBinaryCodecException::TryFromSliceError
    }
}

impl From<serde_json::Error> for XRPLBinaryCodecException {
    fn from(_: serde_json::Error) -> Self {
        XRPLBinaryCodecException::SerdeJsonError
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}
