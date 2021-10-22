//! General XRPL Binary Codec Exceptions.

#[derive(Debug, Clone)]
pub enum XRPLBinaryCodecException {
    UnexpectedParserSkipOverflow { max: usize, found: usize },
    UnexpectedLengthPrefixRange { min: usize, max: usize },
    UnexpectedTypeCodeRange { min: usize, max: usize },
    UnexpectedFieldCodeRange { min: usize, max: usize },
    UnexpectedFieldIdByteRange { min: usize, max: usize },
    UnsupportedCurrencyRepresentation,
    UnknownFieldName,
    InvalidVector256Bytes,
    InvalidReadFromBytesValue,
    InvalidVariableLengthTooLarge { max: usize },
    InvalidHashLength { expected: usize, found: usize },
    InvalidPathSetFromValue,
    HexError(hex::FromHexError),
    SerdeJsonError(serde_json::error::Category),
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

impl alloc::fmt::Display for XRPLBinaryCodecException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLBinaryCodecException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}
