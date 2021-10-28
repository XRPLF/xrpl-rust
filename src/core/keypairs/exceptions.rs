//! XRPL keypair codec exceptions.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;

#[derive(Debug, PartialEq)]
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

impl core::fmt::Display for XRPLKeypairsException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPLKeypairsException: {:?}", self)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLKeypairsException {}
