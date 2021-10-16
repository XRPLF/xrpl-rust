//! Ed25519 elliptic curve cryptography interface.

use crate::constants::CryptoAlgorithm;
use crate::core::keypairs::crypto_implementation::CryptoImplementation;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::sha512_first_half;
use alloc::format;
use alloc::string::String;
use ed25519_dalek::Verifier;
use ed25519_dalek::SIGNATURE_LENGTH;
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey, Signature};

/// ED25519 prefix
pub const ED_PREFIX: &str = "ED";

/// Methods for using the Ed25519 cryptographic system.
pub struct Ed25519;

impl Ed25519 {
    fn _public_key_to_str(key: PublicKey) -> String {
        hex::encode(key.as_ref())
    }

    fn _private_key_to_str(key: SecretKey) -> String {
        hex::encode(key)
    }

    fn _format_key(keystr: &str) -> String {
        format!("{}{}", ED_PREFIX, keystr.to_uppercase())
    }

    fn _format_keys(public: PublicKey, private: SecretKey) -> (String, String) {
        (
            Ed25519::_format_key(&Ed25519::_public_key_to_str(public)),
            Ed25519::_format_key(&Ed25519::_private_key_to_str(private)),
        )
    }
}

impl CryptoImplementation for Ed25519 {
    fn derive_keypair(
        &self,
        decoded_seed: &[u8],
        is_validator: bool,
    ) -> Result<(String, String), XRPLKeypairsException> {
        if is_validator {
            Err(XRPLKeypairsException::UnsupportedValidatorAlgorithm {
                expected: CryptoAlgorithm::ED25519,
            })
        } else {
            let raw_private = sha512_first_half(decoded_seed);
            let private = SecretKey::from_bytes(&raw_private)?;
            let public = PublicKey::from(&private);

            Ok(Ed25519::_format_keys(public, private))
        }
    }

    fn sign(
        &self,
        message: &[u8],
        private_key: &str,
    ) -> Result<[u8; SIGNATURE_LENGTH], XRPLKeypairsException> {
        let raw_private = hex::decode(&private_key[ED_PREFIX.len()..])?;
        let private = SecretKey::from_bytes(&raw_private)?;
        let expanded_private = ExpandedSecretKey::from(&private);
        let public = PublicKey::from(&private);
        let signature: Signature = expanded_private.sign(message, &public);

        Ok(signature.to_bytes())
    }

    fn is_valid_message(
        &self,
        message: &[u8],
        signature: [u8; SIGNATURE_LENGTH],
        public_key: &str,
    ) -> bool {
        let raw_public = hex::decode(&public_key[ED_PREFIX.len()..]);

        if raw_public.is_err() {
            return false;
        };

        let public = PublicKey::from_bytes(&raw_public.unwrap());

        if let Ok(value) = public {
            value.verify(message, &Signature::from(signature)).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::test_cases::PUBLIC_ED25519;
    use crate::core::keypairs::test_cases::RAW_PRIVATE_ED25519;
    use crate::core::keypairs::test_cases::RAW_PUBLIC_ED25519;
    use crate::core::keypairs::test_cases::SEED_ED25519;
    use crate::core::keypairs::test_cases::SIGNATURE_ED25519;
    use crate::core::keypairs::test_cases::TEST_MESSAGE;

    #[test]
    fn test_ed25519_derive_keypair() {
        let (public, private) = Ed25519
            .derive_keypair(SEED_ED25519.as_bytes(), false)
            .unwrap();

        assert_eq!(RAW_PRIVATE_ED25519, public);
        assert_eq!(RAW_PUBLIC_ED25519, private);
    }

    #[test]
    fn test_ed25519_sign() {
        let success = Ed25519.sign(TEST_MESSAGE.as_bytes(), RAW_PRIVATE_ED25519);
        let error = Ed25519.sign(TEST_MESSAGE.as_bytes(), "abc123");

        assert!(success.is_ok());
        assert!(error.is_err());
    }

    #[test]
    fn test_ed25519_is_valid_message() {
        assert!(Ed25519.is_valid_message(
            TEST_MESSAGE.as_bytes(),
            SIGNATURE_ED25519,
            PUBLIC_ED25519
        ))
    }
}
