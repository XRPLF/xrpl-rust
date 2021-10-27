//! Exception for invalid XRP Ledger type data.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::exceptions::ISOCodeException;
use crate::utils::exceptions::JSONParseException;
use crate::utils::exceptions::XRPRangeException;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPLTypeException {
    InvalidNoneValue,
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    XRPLHashError(XRPLHashException),
    XRPLRangeError(XRPRangeException),
    DecimalError(rust_decimal::Error),
    JSONParseError(JSONParseException),
    HexError(hex::FromHexError),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPLHashException {
    InvalidHashLength { expected: usize, found: usize },
    ISOCodeError(ISOCodeException),
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    XRPLAddressCodecError(XRPLAddressCodecException),
    SerdeJsonError(serde_json::error::Category),
    HexError(hex::FromHexError),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum XRPLVectorException {
    InvalidVector256Bytes,
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    XRPLHashError(XRPLHashException),
    HexError(hex::FromHexError),
}

impl From<XRPLHashException> for XRPLTypeException {
    fn from(err: XRPLHashException) -> Self {
        XRPLTypeException::XRPLHashError(err)
    }
}

impl From<XRPRangeException> for XRPLTypeException {
    fn from(err: XRPRangeException) -> Self {
        XRPLTypeException::XRPLRangeError(err)
    }
}

impl From<XRPLBinaryCodecException> for XRPLTypeException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        XRPLTypeException::XRPLBinaryCodecError(err)
    }
}

impl From<JSONParseException> for XRPLTypeException {
    fn from(err: JSONParseException) -> Self {
        XRPLTypeException::JSONParseError(err)
    }
}

impl From<rust_decimal::Error> for XRPLTypeException {
    fn from(err: rust_decimal::Error) -> Self {
        XRPLTypeException::DecimalError(err)
    }
}

impl From<hex::FromHexError> for XRPLTypeException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLTypeException::HexError(err)
    }
}

impl From<ISOCodeException> for XRPLHashException {
    fn from(err: ISOCodeException) -> Self {
        XRPLHashException::ISOCodeError(err)
    }
}

impl From<XRPLBinaryCodecException> for XRPLHashException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        XRPLHashException::XRPLBinaryCodecError(err)
    }
}

impl From<XRPLAddressCodecException> for XRPLHashException {
    fn from(err: XRPLAddressCodecException) -> Self {
        XRPLHashException::XRPLAddressCodecError(err)
    }
}

impl From<serde_json::Error> for XRPLHashException {
    fn from(err: serde_json::Error) -> Self {
        XRPLHashException::SerdeJsonError(err.classify())
    }
}

impl From<hex::FromHexError> for XRPLHashException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLHashException::HexError(err)
    }
}

impl From<XRPLHashException> for XRPLVectorException {
    fn from(err: XRPLHashException) -> Self {
        XRPLVectorException::XRPLHashError(err)
    }
}

impl core::fmt::Display for XRPLTypeException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLTypeException: {:?}", self)
    }
}

impl core::fmt::Display for XRPLHashException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLHashException: {:?}", self)
    }
}

impl core::fmt::Display for XRPLVectorException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLVectorException: {:?}", self)
    }
}
