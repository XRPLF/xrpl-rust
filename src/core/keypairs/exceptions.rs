//! XRPL keypair codec exceptions.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;

#[derive(Debug)]
#[non_exhaustive]
pub enum XRPLKeypairsException {
    InvalidSignature,
    UnsupportedValidatorAlgorithm { expected: CryptoAlgorithm },
    AddressCodecException(XRPLAddressCodecException),
    ED25519Error(ed25519_dalek::ed25519::Error),
    SECP256K1Error(secp256k1::Error),
    HexError(hex::FromHexError),
}

impl From<XRPLAddressCodecException> for XRPLKeypairsException {
    fn from(err: XRPLAddressCodecException) -> Self {
        XRPLKeypairsException::AddressCodecException(err)
    }
}

impl From<ed25519_dalek::ed25519::Error> for XRPLKeypairsException {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        XRPLKeypairsException::ED25519Error(err)
    }
}

impl From<secp256k1::Error> for XRPLKeypairsException {
    fn from(err: secp256k1::Error) -> Self {
        XRPLKeypairsException::SECP256K1Error(err)
    }
}

impl From<hex::FromHexError> for XRPLKeypairsException {
    fn from(err: hex::FromHexError) -> Self {
        XRPLKeypairsException::HexError(err)
    }
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLKeypairsException {}

#[cfg(feature = "std")]
impl alloc::fmt::Display for XRPLKeypairsException {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "XRPLKeypairsException: {:?}", self)
    }
}
