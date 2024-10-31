//! XRPL keypair codec exceptions.

use thiserror_no_std::Error;

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;

#[derive(Debug, PartialEq, Error)]
#[non_exhaustive]
pub enum XRPLKeypairsException {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid secret")]
    InvalidSecret,
    #[error("Unsupported validator algorithm: {expected:?}")]
    UnsupportedValidatorAlgorithm { expected: CryptoAlgorithm },
    #[error("ed25519 error")]
    ED25519Error,
    #[error("secp256k1 error: {0:?}")]
    SECP256K1Error(#[from] secp256k1::Error),
    #[error("XRPL Address codec error: {0}")]
    XRPLAddressCodecError(XRPLAddressCodecException),
}

impl From<XRPLAddressCodecException> for XRPLKeypairsException {
    fn from(err: XRPLAddressCodecException) -> Self {
        XRPLKeypairsException::XRPLAddressCodecError(err)
    }
}

impl From<ed25519_dalek::ed25519::Error> for XRPLKeypairsException {
    fn from(_: ed25519_dalek::ed25519::Error) -> Self {
        XRPLKeypairsException::ED25519Error
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLKeypairsException {}
