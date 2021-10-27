//! Ed25519 elliptic curve cryptography interface.
//! SECP256K1 elliptic curve cryptography interface.
//!
//! Note: The process for using SECP256k1 is complex and
//! more involved than ED25519.
//!
//! See SECP256K1 Key Derivation:
//! `<https://xrpl.org/cryptographic-keys.html#secp256k1-key-derivation>`

use crate::constants::CryptoAlgorithm;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::*;
use crate::core::keypairs::CryptoImplementation;
use alloc::format;
use alloc::string::String;
use core::convert::TryInto;
use core::str::FromStr;
use ed25519_dalek::Verifier;
use num_bigint::BigUint;
use rust_decimal::prelude::One;
use secp256k1::constants::CURVE_ORDER;
use secp256k1::SignOnly;
use secp256k1::VerifyOnly;

/// MMethods for using the ECDSA cryptographic system with
/// the SECP256K1 elliptic curve.
pub struct Secp256k1;

/// Methods for using the ED25519 cryptographic system.
pub struct Ed25519;

/// TODO Not working
impl Secp256k1 {
    /// Hex encode the private key.
    fn _private_key_to_str(key: secp256k1::SecretKey) -> String {
        hex::encode_upper(key.as_ref())
    }

    /// Hex encode the public key.
    fn _public_key_to_str(key: secp256k1::PublicKey) -> String {
        hex::encode_upper(key.serialize())
    }

    /// Format a provided key.
    fn _format_key(keystr: &str) -> String {
        let padding = SECP256K1_KEY_LENGTH - keystr.len();
        format!("{:0<pad$}", keystr.to_uppercase(), pad = padding)
    }

    /// Format the public and private keys.
    fn _format_keys(
        public: secp256k1::PublicKey,
        private: secp256k1::SecretKey,
    ) -> (String, String) {
        (
            Secp256k1::_format_key(&Secp256k1::_public_key_to_str(public)),
            Secp256k1::_format_key(&Secp256k1::_private_key_to_str(private)),
        )
    }

    /// Determing if the provided secret key is valid.
    fn _is_secret_valid(key: secp256k1::SecretKey) -> bool {
        let key_bytes = BigUint::from_bytes_be(key.as_ref());
        key_bytes >= BigUint::one() && key_bytes <= BigUint::from_bytes_be(&CURVE_ORDER)
    }

    //fn _get_secret()
}

impl Ed25519 {
    /// Hex encode the private key.
    fn _private_key_to_str(key: ed25519_dalek::SecretKey) -> String {
        hex::encode(key)
    }

    /// Hex encode the public key.
    fn _public_key_to_str(key: ed25519_dalek::PublicKey) -> String {
        hex::encode(key.as_ref())
    }

    /// Format a provided key.
    fn _format_key(keystr: &str) -> String {
        format!("{}{}", ED25519_PREFIX, keystr.to_uppercase())
    }

    /// Format the public and private keys.
    fn _format_keys(
        public: ed25519_dalek::PublicKey,
        private: ed25519_dalek::SecretKey,
    ) -> (String, String) {
        (
            Ed25519::_format_key(&Ed25519::_public_key_to_str(public)),
            Ed25519::_format_key(&Ed25519::_private_key_to_str(private)),
        )
    }
}

impl CryptoImplementation for Secp256k1 {
    fn derive_keypair(
        &self,
        decoded_seed: &[u8],
        _is_validator: bool,
    ) -> Result<(String, String), XRPLKeypairsException> {
        let secp = secp256k1::Secp256k1::new();
        let secret_key = secp256k1::SecretKey::from_slice(decoded_seed)?;
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Secp256k1::_format_keys(public_key, secret_key))
    }

    fn sign(
        &self,
        message_bytes: &[u8],
        private_key: &str,
    ) -> Result<String, XRPLKeypairsException> {
        let secp = secp256k1::Secp256k1::<SignOnly>::signing_only();
        let message = secp256k1::Message::from_slice(message_bytes)?;
        let private = secp256k1::SecretKey::from_str(private_key)?;
        let signature = secp.sign(&message, &private);

        Ok(hex::encode_upper(signature.serialize_compact()))
    }

    fn is_valid_message(&self, message_bytes: &[u8], signature: &str, public_key: &str) -> bool {
        let secp = secp256k1::Secp256k1::<VerifyOnly>::verification_only();
        let msg = secp256k1::Message::from_slice(message_bytes);
        let signature_compact = hex::decode(signature);

        if let Ok(compact) = signature_compact {
            let sig = secp256k1::Signature::from_compact(&compact);
            let public = secp256k1::PublicKey::from_str(public_key);

            if let (&Ok(m), &Ok(s), &Ok(p)) = (&msg.as_ref(), &sig.as_ref(), &public.as_ref()) {
                secp.verify(m, s, p).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl CryptoImplementation for Ed25519 {
    /// Derives a key pair for use with the XRP Ledger
    /// from a seed value.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::keypairs::algorithms::Ed25519;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let decoded_seed: &[u8] = &[
    ///     207, 45, 227, 120, 251, 221, 126, 46,
    ///     232, 125, 72, 109, 251, 90, 123, 255
    /// ];
    /// let validator: bool = false;
    /// let tuple: (String, String) = (
    ///     "ED60292139838CB86E719134F848F055057CA5BDA61F5A529729F1697502D53E1C".into(),
    ///     "ED009F66528611A0D400946A01FA01F8AF4FF4C1D0C744AE3F193317DCA77598F1".into(),
    /// );
    ///
    /// let derivation: Option<(String, String)> = match Ed25519.derive_keypair(
    ///     decoded_seed,
    ///     validator,
    /// ) {
    ///     Ok((public, private)) => Some((public, private)),
    ///     Err(e) => match e {
    ///         XRPLKeypairsException::ED25519Error(_ed25519_error) => None,
    ///         XRPLKeypairsException::UnsupportedValidatorAlgorithm { expected: _ } => None,
    ///         _ => None,
    ///     },
    /// };
    ///
    /// assert_eq!(Some(tuple), derivation);
    /// ```
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
            let private = ed25519_dalek::SecretKey::from_bytes(&raw_private)?;
            let public = ed25519_dalek::PublicKey::from(&private);

            Ok(Ed25519::_format_keys(public, private))
        }
    }

    /// Signs a message using a given private key.
    /// * `message` - Text about foo.
    /// * `private_key` - Text about bar.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::keypairs::algorithms::Ed25519;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let message: &[u8] = "test message".as_bytes();
    /// let private_key: &str = "EDB4C4E046826BD26190D09715FC31F4E\
    ///                          6A728204EADD112905B08B14B7F15C4F3";
    /// let signature: String = "CB199E1BFD4E3DAA105E4832EEDFA3641\
    ///                          3E1F44205E4EFB9E27E826044C21E3E2E\
    ///                          848BBC8195E8959BADF887599B7310AD1\
    ///                          B7047EF11B682E0D068F73749750E".into();
    ///
    /// let signing: Option<String> = match Ed25519.sign(
    ///     message,
    ///     private_key,
    /// ) {
    ///     Ok(signature) => Some(signature),
    ///     Err(e) => match e {
    ///         XRPLKeypairsException::ED25519Error(_ed25519_error) => None,
    ///         _ => None,
    ///     },
    /// };
    ///
    /// assert_eq!(Some(signature), signing);
    /// ```
    fn sign(&self, message: &[u8], private_key: &str) -> Result<String, XRPLKeypairsException> {
        let raw_private = hex::decode(&private_key[ED25519_PREFIX.len()..])?;
        let private = ed25519_dalek::SecretKey::from_bytes(&raw_private)?;
        let expanded_private = ed25519_dalek::ExpandedSecretKey::from(&private);
        let public = ed25519_dalek::PublicKey::from(&private);
        let signature: ed25519_dalek::Signature = expanded_private.sign(message, &public);

        Ok(hex::encode_upper(signature.to_bytes()))
    }

    /// Verifies the signature on a given message.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::keypairs::algorithms::Ed25519;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let message: &[u8] = "test message".as_bytes();
    /// let signature: &str = "CB199E1BFD4E3DAA105E4832EEDFA3641\
    ///                        3E1F44205E4EFB9E27E826044C21E3E2E\
    ///                        848BBC8195E8959BADF887599B7310AD1\
    ///                        B7047EF11B682E0D068F73749750E";
    /// let public_key: &str = "ED01FA53FA5A7E77798F882ECE20B1AB\
    ///                         C00BB358A9E55A202D0D0676BD0CE37A63";
    ///
    /// assert!(Ed25519.is_valid_message(
    ///     message,
    ///     signature,
    ///     public_key,
    /// ));
    /// ```
    fn is_valid_message(&self, message: &[u8], signature: &str, public_key: &str) -> bool {
        let raw_public = hex::decode(&public_key[ED25519_PREFIX.len()..]);
        let decoded_sig = hex::decode(signature);

        if raw_public.is_err() || decoded_sig.is_err() {
            return false;
        };

        let bytes = decoded_sig.unwrap();
        let public = ed25519_dalek::PublicKey::from_bytes(&raw_public.unwrap());

        if bytes.len() != ED25519_SIGNATURE_LENGTH {
            return false;
        };

        if let Ok(value) = public {
            let sig: [u8; ED25519_SIGNATURE_LENGTH] = bytes.try_into().expect("is_valid_message");
            let converted = &ed25519_dalek::Signature::from(sig);

            value.verify(message, converted).is_ok()
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
        let signature: &str = &hex::encode_upper(SIGNATURE_ED25519);
        let message: &[u8] = TEST_MESSAGE.as_bytes();

        assert!(Ed25519.is_valid_message(message, signature, PUBLIC_ED25519));
    }
}
