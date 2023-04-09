//! General XRPL Address Codec Exception.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::exceptions::ISOCodeException;
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Display)]
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
    FromHexError,
    Base58DecodeError(bs58::decode::Error),
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
    fn from(_: hex::FromHexError) -> Self {
        XRPLAddressCodecException::FromHexError
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

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLAddressCodecException {}
