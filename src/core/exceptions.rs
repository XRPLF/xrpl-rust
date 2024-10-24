use thiserror_no_std::Error;

use crate::{utils::exceptions::XRPRangeException, XRPLSerdeJsonError};

use super::{
    addresscodec::exceptions::XRPLAddressCodecException,
    binarycodec::{
        exceptions::XRPLBinaryCodecException,
        types::exceptions::{XRPLHashException, XRPLTypeException},
    },
    keypairs::exceptions::XRPLKeypairsException,
};

pub type XRPLCoreResult<T> = core::result::Result<T, XRPLCoreException>;

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
}

impl From<XRPLTypeException> for XRPLCoreException {
    fn from(err: XRPLTypeException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(err))
    }
}

impl From<XRPLHashException> for XRPLCoreException {
    fn from(err: XRPLHashException) -> Self {
        XRPLCoreException::XRPLBinaryCodecError(XRPLBinaryCodecException::XRPLTypeError(
            XRPLTypeException::XRPLHashError(err),
        ))
    }
}
