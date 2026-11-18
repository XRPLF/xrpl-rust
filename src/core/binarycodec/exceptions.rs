//! General XRPL Binary Codec Exceptions.

use crate::utils::exceptions::XRPRangeException;

use super::types::exceptions::XRPLTypeException;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Error)]
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
    #[error("Empty path in path set")]
    EmptyPathInPathSet,
    #[error("Unknown path step type byte: {type_byte:#04X}")]
    UnknownPathStepType { type_byte: u8 },
    #[error("Mutually exclusive path step fields in type byte {type_byte:#04X}: {first} and {second} cannot both be present")]
    MutuallyExclusivePathStepFields {
        type_byte: u8,
        first: &'static str,
        second: &'static str,
    },
    #[error("Try from slice error")]
    TryFromSliceError,
    #[error("Field has no associated tag")]
    FieldHasNoAssiciatedTag,
    #[error("XAddress tag mismatch")]
    XAddressTagMismatch,
    #[error("Field is not account or destination")]
    FieldIsNotAccountOrDestination,
    #[error("Try from int error: {0}")]
    TryFromIntError(#[from] core::num::TryFromIntError),
    #[error("From utf8 error: {0}")]
    FromUtf8Error(#[from] alloc::string::FromUtf8Error),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),
    #[error("XRPL Type error: {0}")]
    XRPLTypeError(#[from] XRPLTypeException),
    #[error("XRP Range error: {0}")]
    XRPRangeError(#[from] XRPRangeException),
}

impl From<core::array::TryFromSliceError> for XRPLBinaryCodecException {
    fn from(_: core::array::TryFromSliceError) -> Self {
        XRPLBinaryCodecException::TryFromSliceError
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}
