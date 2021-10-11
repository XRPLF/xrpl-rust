//! Ed25519 elliptic curve cryptography interface.

use crate::core::keypairs::crypto_implementation::CryptoImplementation;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::sha512_first_half;
use alloc::format;
use alloc::string::String;
use core::convert::TryInto;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey, Sha512, Signature, Verifier};

const PREFIX: &str = "ED";

/// Methods for using the Ed25519 cryptographic system.
pub struct Ed25519;

impl Ed25519 {
    fn _public_key_to_str(&self, key: PublicKey) -> String {
        let curve: EdwardsPoint = EdwardsPoint::hash_from_bytes::<Sha512>(key.as_bytes());
        let compressed: CompressedEdwardsY = curve.compress();

        hex::encode(compressed.0)
    }

    fn _private_key_to_str(&self, key: SecretKey) -> String {
        hex::encode(key)
    }

    fn _format_key(&self, keystr: &str) -> String {
        format!("{}{}", PREFIX, keystr.to_uppercase())
    }
}

impl CryptoImplementation for Ed25519 {
    fn derive_keypair(
        &self,
        decoded_seed: &[u8],
        is_validator: bool,
    ) -> Result<(String, String), XRPLKeypairsException> {
        if is_validator {
            Err(XRPLKeypairsException::new(
                "Validator key pairs cannot use ED25519",
            ))
        } else {
            let raw_private = sha512_first_half(decoded_seed);
            alloc::println!("{:?}", raw_private);
            let private = SecretKey::from_bytes(&raw_private)?;
            let public = PublicKey::from(&private);

            Ok((
                self._format_key(&self._public_key_to_str(public)),
                self._format_key(&self._private_key_to_str(private)),
            ))
        }
    }

    fn sign(&self, message: &[u8], private_key: &str) -> Result<[u8; 64], XRPLKeypairsException> {
        let raw_private = hex::decode(&private_key[PREFIX.len()..])?;
        let private = SecretKey::from_bytes(&raw_private)?;
        let expanded_private = ExpandedSecretKey::from(&private);
        let public = PublicKey::from(&private);

        let signature: Signature = expanded_private.sign(message, &public);

        Ok(signature.to_bytes())
    }

    fn is_valid_message(&self, message: &[u8], signature: [u8; 64], public_key: &str) -> bool {
        let raw_public = hex::decode(&public_key[PREFIX.len()..]);

        if raw_public.is_err() {
            return false;
        };

        let bytes: [u8; 32] = raw_public.unwrap().try_into().expect("Invalid length");
        let compressed: CompressedEdwardsY = CompressedEdwardsY(bytes);
        let public = PublicKey::from_bytes(&compressed.0);

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
    use crate::core::keypairs::main_test_cases::PRIVATE_ED25519;
    use crate::core::keypairs::main_test_cases::PUBLIC_ED25519;
    use crate::core::keypairs::main_test_cases::SEED;

    #[test]
    fn test_ed25519_derive_keypair() {
        let (public, private) = Ed25519.derive_keypair(SEED.as_bytes(), false).unwrap();

        assert_eq!(PRIVATE_ED25519, private);
        assert_eq!(PUBLIC_ED25519, public);
    }

    #[test]
    fn test_ed25519_sign() {
        let success = Ed25519.sign(b"Hello World!", PRIVATE_ED25519);
        let error = Ed25519.sign(b"Hello World!", "abc123");

        assert!(success.is_ok());
        assert!(error.is_err());
    }

    #[test]
    fn test_ed25519_is_valid_message() {
        let (public, private) = Ed25519
            .derive_keypair(b"sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r", false)
            .unwrap();
    }
}
