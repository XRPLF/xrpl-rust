//! General XRPL Binary Codec Exceptions.

#[derive(Debug, Clone)]
pub enum XRPLBinaryCodecException {
    UnexpectedParserSkipOverflow { max: usize, found: usize },
    UnexpectedLengthPrefixRange { min: usize, max: usize },
    UnexpectedTypeCodeRange { min: usize, max: usize },
    UnexpectedFieldCodeRange { min: usize, max: usize },
    UnknownFieldName,
    InvalidReadFromBytesValue,
    InvalidVariableLengthTooLarge { max: usize },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLBinaryCodecException {}

#[cfg(feature = "std")]
impl alloc::fmt::Display for XRPLBinaryCodecException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPLBinaryCodecException: {:?}", self)
    }
}
