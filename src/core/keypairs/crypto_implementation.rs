//! Traits for cryptographic algorithms in the XRP Ledger.

/// Trait for cryptographic algorithms in the XRP Ledger.
/// The classes for all cryptographic algorithms are
/// derived from this trait.
trait CryptoImplementation {
    fn derive_keypair(&self, decoded_seed: &[u8], is_validator: bool) -> (String, String);
    fn sign(&self, message: &[u8], private_key: &str) -> Vec<u8>;
    fn is_valid_message(&self, message: &[u8], signature: &[u8], public_key: &str) -> bool;
}
