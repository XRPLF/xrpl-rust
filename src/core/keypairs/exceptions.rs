//! XRPL keypair codec exceptions.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::mutate_from_error;
use alloc::borrow::Cow;
use alloc::string::ToString;

/// General XRPL Keypair Codec Exception.
#[derive(Debug)]
pub struct XRPLKeypairsException(Cow<'static, str>);

impl XRPLKeypairsException {
    pub fn new(err: &str) -> XRPLKeypairsException {
        XRPLKeypairsException(err.to_string().into())
    }
}

mutate_from_error!(ed25519_dalek::ed25519::Error, XRPLKeypairsException);
mutate_from_error!(XRPLAddressCodecException, XRPLKeypairsException);
mutate_from_error!(hex::FromHexError, XRPLKeypairsException);
mutate_from_error!(secp256k1::Error, XRPLKeypairsException);
