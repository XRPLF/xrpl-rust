//! XRPL keypair codec exceptions.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use strum_macros::Display;

#[derive(Debug, Clone, PartialEq, Display)]
#[non_exhaustive]
pub enum XRPLKeypairsException {
    InvalidSignature,
    InvalidSecret,
    UnsupportedValidatorAlgorithm { expected: CryptoAlgorithm },
    ED25519Error,
    SECP256K1Error,
    FromHexError,
    AddressCodecException(XRPLAddressCodecException),
}

impl From<XRPLAddressCodecException> for XRPLKeypairsException {
    fn from(err: XRPLAddressCodecException) -> Self {
        XRPLKeypairsException::AddressCodecException(err)
    }
}

impl From<ed25519_dalek::ed25519::Error> for XRPLKeypairsException {
    fn from(_: ed25519_dalek::ed25519::Error) -> Self {
        XRPLKeypairsException::ED25519Error
    }
}

impl From<secp256k1::Error> for XRPLKeypairsException {
    fn from(_: secp256k1::Error) -> Self {
        XRPLKeypairsException::SECP256K1Error
    }
}

impl From<hex::FromHexError> for XRPLKeypairsException {
    fn from(_: hex::FromHexError) -> Self {
        XRPLKeypairsException::FromHexError
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLKeypairsException {}
