//! Traits for cryptographic algorithms in the XRP Ledger.

use crate::core::keypairs::exceptions::XRPLKeypairsException;
use alloc::string::String;

/// Trait for cryptographic algorithms in the XRP Ledger.
/// The classes for all cryptographic algorithms are
/// derived from this trait.
pub trait CryptoImplementation {
    /// Derives a key pair for use with the XRP Ledger
    /// from a seed value.
    fn derive_keypair(
        &self,
        decoded_seed: &[u8],
        is_validator: bool,
    ) -> Result<(String, String), XRPLKeypairsException>;
    /// Signs a message using a given private key.
    /// * `message` - Text about foo.
    /// * `private_key` - Text about bar.
    fn sign(&self, message: &[u8], private_key: &str) -> Result<[u8; 64], XRPLKeypairsException>;
    /// Verifies the signature on a given message.
    fn is_valid_message(&self, message: &[u8], signature: [u8; 64], public_key: &str) -> bool;
}
