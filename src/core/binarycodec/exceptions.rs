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
    InvalidReadFromBytesValue,
    InvalidVariableLengthTooLarge { max: usize },
    InvalidHashLength { expected: usize, found: usize },
    HexError(hex::FromHexError),
}

impl From<hex::FromHexError> for XRPLBinaryCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLBinaryCodecException::HexError(err)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}

#[cfg(feature = "std")]
impl alloc::fmt::Display for XRPLBinaryCodecException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPLBinaryCodecException: {:?}", self)
    }
}
