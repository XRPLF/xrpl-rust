//! General XRPL Address Codec Exception.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::exceptions::ISOCodeException;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLAddressCodecException {
    #[error("Invalid XAddress prefix")]
    InvalidXAddressPrefix,
    #[error("Invalid XAddress zero tag")]
    InvalidXAddressZeroNoTag,
    #[error("Invalid XAddress zero remain")]
    InvalidXAddressZeroRemain,
    #[error("Invalid classic address length (length: {length})")]
    InvalidCAddressIdLength { length: usize },
    #[error("Invalid classic address tag")]
    InvalidCAddressTag,
    #[error("Invalid seed prefix encoding type")]
    InvalidSeedPrefixEncodingType,
    #[error("Invalid encoding prefix length")]
    InvalidEncodingPrefixLength,
    #[error("Invalid classic address value")]
    InvalidClassicAddressValue,
    #[error("Unsupported XAddress")]
    UnsupportedXAddress,
    #[error("Unknown seed encoding")]
    UnknownSeedEncoding,
    #[error("Unknown payload lenght (expected: {expected}, found: {found})")]
    UnexpectedPayloadLength { expected: usize, found: usize },
    #[error("From hex error")]
    FromHexError,
    #[error("Base58 decode error: {0}")]
    Base58DecodeError(bs58::decode::Error),
    #[error("XRPL binary codec error: {0}")]
    XRPLBinaryCodecError(XRPLBinaryCodecException),
    #[error("ISO code error: {0}")]
    ISOError(ISOCodeException),
    #[error("Serde json error")]
    SerdeJsonError(serde_json::error::Category),
    #[error("Vec resize error")]
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
