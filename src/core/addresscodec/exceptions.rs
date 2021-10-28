//! General XRPL Address Codec Exception.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::exceptions::ISOCodeException;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum XRPLAddressCodecException {
    InvalidXAddressPrefix,
    InvalidXAddressZeroNoTag,
    InvalidXAddressZeroRemain,
    InvalidCAddressIdLength { length: usize },
    InvalidCAddressTag,
    InvalidSeedPrefixEncodingType,
    InvalidEncodingPrefixLength,
    InvalidClassicAddressValue,
    UnsupportedXAddress,
    UnknownSeedEncoding,
    UnexpectedPayloadLength { expected: usize, found: usize },
    Base58DecodeError(bs58::decode::Error),
    HexError(hex::FromHexError),
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    ISOError(ISOCodeException),
    SerdeJsonError(serde_json::error::Category),
    VecResizeError(alloc::vec::Vec<u8>),
}

impl From<ISOCodeException> for XRPLAddressCodecException {
    fn from(err: ISOCodeException) -> Self {
        XRPLAddressCodecException::ISOError(err)
    }
}

impl From<XRPLBinaryCodecException> for XRPLAddressCodecException {
    fn from(err: XRPLBinaryCodecException) -> Self {
        XRPLAddressCodecException::XRPLBinaryCodecError(err)
    }
}

impl From<bs58::decode::Error> for XRPLAddressCodecException {
    fn from(err: bs58::decode::Error) -> Self {
        XRPLAddressCodecException::Base58DecodeError(err)
    }
}

impl From<hex::FromHexError> for XRPLAddressCodecException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLAddressCodecException::HexError(err)
    }
}

impl From<serde_json::Error> for XRPLAddressCodecException {
    fn from(err: serde_json::Error) -> Self {
        XRPLAddressCodecException::SerdeJsonError(err.classify())
    }
}

impl From<alloc::vec::Vec<u8>> for XRPLAddressCodecException {
    fn from(err: alloc::vec::Vec<u8>) -> Self {
        XRPLAddressCodecException::VecResizeError(err)
    }
}

impl core::fmt::Display for XRPLAddressCodecException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(f, "XRPLAddressCodecException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLAddressCodecException {}
