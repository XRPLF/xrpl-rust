//! SECP256K1 elliptic curve cryptography interface.
//! The process for using SECP256k1 is complex and more
//! involved than ED25519.
//!
//! See: `<https://xrpl.org/cryptographic-keys.html#secp256k1-key-derivation>`

use crate::core::keypairs::crypto_implementation::CryptoImplementation;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use alloc::format;
use alloc::string::String;
use core::str::FromStr;
use num_bigint::BigUint;
use rust_decimal::prelude::One;
use secp256k1::constants::CURVE_ORDER;
use secp256k1::SignOnly;
use secp256k1::VerifyOnly;
use secp256k1::{Message, PublicKey, SecretKey, Signature};

/// String keys must be _KEY_LENGTH long
const _KEY_LENGTH: usize = 66;

/// Generated sequence values are _SEQUENCE_SIZE bytes
/// unsigned big-endian
const _SEQUENCE_SIZE: usize = 4;
const _SEQUENCE_MAX: usize = usize::pow(256, _SEQUENCE_SIZE as u32);

/// Intermediate private keys are always padded with
/// 4 bytes of zeros.
const _INTERMEDIATE_KEYPAIR_PADDING: [u8; 4] = [0, 0, 0, 0];

/// MMethods for using the ECDSA cryptographic system with
/// the secp256k1 elliptic curve.
pub struct Secp256k1;

impl Secp256k1 {
    fn _private_key_to_str(key: SecretKey) -> String {
        hex::encode(key.as_ref())
    }

    fn _public_key_to_str(key: PublicKey) -> String {
        hex::encode(key.serialize())
    }

    fn _format_key(keystr: &str) -> String {
        let padding = _KEY_LENGTH - keystr.len();
        format!("{:0<pad$}", keystr.to_uppercase(), pad = padding)
    }

    fn _format_keys(public: PublicKey, private: SecretKey) -> (String, String) {
        (
            Secp256k1::_format_key(&Secp256k1::_public_key_to_str(public)),
            Secp256k1::_format_key(&Secp256k1::_private_key_to_str(private)),
        )
    }

    fn _is_secret_valid(key: SecretKey) -> bool {
        let key_bytes = BigUint::from_bytes_be(key.as_ref());
        key_bytes >= BigUint::one() && key_bytes <= BigUint::from_bytes_be(&CURVE_ORDER)
    }

    //fn _get_secret()
}

impl CryptoImplementation for Secp256k1 {
    fn derive_keypair(
        &self,
        decoded_seed: &[u8],
        _is_validator: bool,
    ) -> Result<(String, String), XRPLKeypairsException> {
        let secp = secp256k1::Secp256k1::new();
        let secret_key = SecretKey::from_slice(decoded_seed)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Secp256k1::_format_keys(public_key, secret_key))
    }

    fn sign(
        &self,
        message_bytes: &[u8],
        private_key: &str,
    ) -> Result<[u8; 64], XRPLKeypairsException> {
        let secp = secp256k1::Secp256k1::<SignOnly>::signing_only();
        let message = Message::from_slice(message_bytes)?;
        let private = SecretKey::from_str(private_key)?;
        let signature = secp.sign(&message, &private);

        Ok(signature.serialize_compact())
    }

    fn is_valid_message(
        &self,
        message_bytes: &[u8],
        signature_compact: [u8; 64],
        public_key: &str,
    ) -> bool {
        let secp = secp256k1::Secp256k1::<VerifyOnly>::verification_only();
        let msg = Message::from_slice(message_bytes);
        let sig = Signature::from_compact(&signature_compact);
        let public = PublicKey::from_str(public_key);

        if let (&Ok(m), &Ok(s), &Ok(p)) = (&msg.as_ref(), &sig.as_ref(), &public.as_ref()) {
            secp.verify(m, s, p).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    // TODO
    // use super::*;
    // use crate::core::keypairs::test_cases::PRIVATE_SECP256K1;
    // use crate::core::keypairs::test_cases::PUBLIC_SECP256K1;
    // use crate::core::keypairs::test_cases::SEED_SECP256K1;
    // use crate::core::keypairs::test_cases::TEST_MESSAGE;

    #[test]
    fn test_secp256k1_derive_keypair() {
        // let (public, private) = Secp256k1
        //     .derive_keypair(SEED_SECP256K1.as_bytes(), false)
        //     .unwrap();

        //assert_eq!(PRIVATE_SECP256K1, public);
        //assert_eq!(PUBLIC_SECP256K1, private);
    }

    #[test]
    fn test_secp256k1_sign() {
        // let success = Secp256k1.sign(TEST_MESSAGE.as_bytes(), PRIVATE_SECP256K1);
        // let error = Secp256k1.sign(TEST_MESSAGE.as_bytes(), "abc123");

        // assert!(success.is_ok());
        // assert!(error.is_err());
    }

    #[test]
    fn test_secp256k1_is_valid_message() {
        // assert!(Secp256k1.is_valid_message(
        //     TEST_MESSAGE.as_bytes(),
        //     SIGNATURE_SECP256K1,
        //     PUBLIC_SECP256K1
        // ))
    }
}
