//! XRPL keypair codec exceptions.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::mutate_from_error;
use alloc::string::String;
use alloc::string::ToString;

fn typed_alert<T>(message: &str) -> String {
    let typed = alloc::any::type_name::<T>();
    alloc::format!("{}: {}", typed, message)
}

/// General XRPL Keypair Codec Exception.
#[derive(Debug)]
pub struct XRPLKeypairsException {
    pub message: String,
}

impl XRPLKeypairsException {
    pub fn new(err: &str) -> XRPLKeypairsException {
        XRPLKeypairsException {
            message: typed_alert::<ed25519_dalek::ed25519::Error>(&err.to_string()),
        }
    }
}

mutate_from_error!(ed25519_dalek::ed25519::Error, XRPLKeypairsException);
mutate_from_error!(XRPLAddressCodecException, XRPLKeypairsException);
mutate_from_error!(hex::FromHexError, XRPLKeypairsException);
