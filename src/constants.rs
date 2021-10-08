//! Collection of public constants for XRPL.

use strum_macros::EnumIter;

/// Represents the supported cryptography algorithms.
#[derive(Debug, PartialEq, Clone, EnumIter)]
pub enum CryptoAlgorithm {
    ED25519,
    SECP256K1,
}

impl std::string::ToString for CryptoAlgorithm {
    fn to_string(&self) -> std::string::String {
        match *self {
            CryptoAlgorithm::ED25519 => "ed25519".to_string(),
            CryptoAlgorithm::SECP256K1 => "secp256k1".to_string(),
        }
    }
}
