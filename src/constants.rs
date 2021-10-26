//! Collection of public constants for XRPL.

use alloc::string::String;
use alloc::string::ToString;
use strum_macros::EnumIter;

/// Regular expression for determining ISO currency codes.
pub const ISO_CURRENCY_REGEX: &str = r"^[A-Z0-9]{3}$";
/// Regular expression for determining hex currency codes.
pub const HEX_CURRENCY_REGEX: &str = r"^[A-F0-9]{40}$";

/// Length of an account id.
pub const ACCOUNT_ID_LENGTH: usize = 20;

/// Represents the supported cryptography algorithms.
#[derive(Debug, PartialEq, Clone, EnumIter)]
pub enum CryptoAlgorithm {
    ED25519,
    SECP256K1,
}

impl ToString for CryptoAlgorithm {
    /// Return the String representation of an algorithm.
    fn to_string(&self) -> String {
        match *self {
            CryptoAlgorithm::ED25519 => "ed25519".to_string(),
            CryptoAlgorithm::SECP256K1 => "secp256k1".to_string(),
        }
    }
}
