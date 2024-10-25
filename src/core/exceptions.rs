use alloc::string::String;
use thiserror_no_std::Error;

use crate::{
    utils::exceptions::{ISOCodeException, XRPLUtilsException},
    XRPLSerdeJsonError,
};

use super::{
    addresscodec::exceptions::XRPLAddressCodecException,
    binarycodec::{
        exceptions::XRPLBinaryCodecException,
        types::exceptions::{
            XRPLHashException, XRPLSerializeArrayException, XRPLSerializeMapException,
            XRPLTypeException, XRPLVectorException, XRPLXChainBridgeException,
        },
    },
    keypairs::exceptions::XRPLKeypairsException,
};

pub type XRPLCoreResult<T, E = XRPLCoreException> = core::result::Result<T, E>;

#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLCoreException {
    #[error("XRPL Address Codec error: {0}")]
    XRPLAddressCodecError(#[from] XRPLAddressCodecException),
    #[error("XRPL Binary Codec error: {0}")]
    XRPLBinaryCodecError(#[from] XRPLBinaryCodecException),
    #[error("XRPL Keypairs error: {0}")]
    XRPLKeypairsError(#[from] XRPLKeypairsException),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] XRPLSerdeJsonError),
    #[error("XRPL utils error: {0}")]
    XRPLUtilsError(String), // TODO: find a better way to avoid infinite recursion
    #[error("From hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
    #[error("ISO code error: {0}")]
    ISOCodeError(#[from] ISOCodeException),
    #[error("Base58 error: {0}")]
    Bs58Error(#[from] bs58::decode::Error),
}

impl From<XRPLTypeException> for XRPLCoreException {
    fn from(error: XRPLTypeException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(error))
    }
}

impl From<XRPLSerializeArrayException> for XRPLCoreException {
    fn from(error: XRPLSerializeArrayException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLSerializeArrayException(error),
        ))
    }
}

impl From<XRPLSerializeMapException> for XRPLCoreException {
    fn from(error: XRPLSerializeMapException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLSerializeMapException(error),
        ))
    }
}

impl From<XRPLXChainBridgeException> for XRPLCoreException {
    fn from(error: XRPLXChainBridgeException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLXChainBridgeError(error),
        ))
    }
}

impl From<XRPLHashException> for XRPLCoreException {
    fn from(error: XRPLHashException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLHashError(error),
        ))
    }
}

impl From<XRPLVectorException> for XRPLCoreException {
    fn from(error: XRPLVectorException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLVectorError(error),
        ))
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLCoreException {}
