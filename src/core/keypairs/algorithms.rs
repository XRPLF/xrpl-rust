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
use alloc::vec::Vec;
use core::convert::TryInto;
use core::str::FromStr;
use ed25519_dalek::Verifier;
use num_bigint::BigUint;
use rust_decimal::prelude::One;

/// Methods for using the ECDSA cryptographic system with
/// the SECP256K1 elliptic curve.
pub struct Secp256k1;

/// Methods for using the ED25519 cryptographic system.
pub struct Ed25519;

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
        format!("{:0>pad$}", keystr, pad = SECP256K1_KEY_LENGTH)
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

    /// Hash the message to prevent insecure signing.
    fn _get_message(message: &[u8]) -> Result<secp256k1::Message, secp256k1::Error> {
        secp256k1::Message::from_slice(&sha512_first_half(message))
    }

    /// Determing if the provided secret key is valid.
    fn _is_secret_valid(key: &[u8]) -> bool {
        let key_bytes = BigUint::from_bytes_be(key);
        key_bytes >= BigUint::one()
            && key_bytes <= BigUint::from_bytes_be(&secp256k1::constants::CURVE_ORDER)
    }

    /// Concat candidate key.
    fn _candidate_merger(input: &[u8], candidate: &[u8], phase: &Secp256k1Phase) -> Vec<u8> {
        if phase == &Secp256k1Phase::Root {
            [input, candidate].concat()
        } else {
            [input, &SECP256K1_INTERMEDIATE_KEYPAIR_PADDING, candidate].concat()
        }
    }

    /// Given bytes_input determine public/private keypair
    /// for a given phase of this algorithm. The difference
    /// between generating the root and intermediate keypairs
    /// is just what bytes are input by the caller and that
    /// the intermediate keypair needs to inject
    /// SECP256K1_INTERMEDIATE_KEYPAIR_PADDING into the value
    /// to hash to get the raw private key.
    fn _derive_part(
        bytes: &[u8],
        phase: Secp256k1Phase,
    ) -> Result<(secp256k1::PublicKey, secp256k1::SecretKey), XRPLKeypairsException> {
        let raw_private = Self::_get_secret(bytes, &phase)?;
        let secp = secp256k1::Secp256k1::new();
        let wrapped_private = secp256k1::SecretKey::from_slice(&raw_private)?;
        let wrapped_public = secp256k1::PublicKey::from_secret_key(&secp, &wrapped_private);

        Ok((wrapped_public, wrapped_private))
    }

    /// Derive the final public/private keys.
    fn _derive_final(
        root_public: secp256k1::PublicKey,
        root_private: secp256k1::SecretKey,
        mid_public: secp256k1::PublicKey,
        mid_private: secp256k1::SecretKey,
    ) -> Result<(secp256k1::PublicKey, secp256k1::SecretKey), XRPLKeypairsException> {
        let mut wrapped_private = root_private;
        let wrapped_public = root_public.combine(&mid_public)?;

        wrapped_private.add_assign(mid_private.as_ref())?;
        Ok((wrapped_public, wrapped_private))
    }

    /// Given a function `candidate_merger` that knows how
    /// to prepare a sequence candidate bytestring into a
    /// possible full candidate secret, returns the first
    /// sequence value that is valid. If none are valid,
    /// raises; however this should be so exceedingly rare
    /// as to ignore.
    fn _get_secret(
        input: &[u8],
        phase: &Secp256k1Phase,
    ) -> Result<[u8; SHA512_HASH_LENGTH], XRPLKeypairsException> {
        for raw_root in 0..SECP256K1_SEQUENCE_MAX {
            let root = (raw_root as u32).to_be_bytes();
            let candidate = sha512_first_half(&Self::_candidate_merger(input, &root, phase));

            if Self::_is_secret_valid(&candidate) {
                return Ok(candidate);
            } else {
                continue;
            }
        }

        Err(XRPLKeypairsException::InvalidSecret)
    }
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
    /// Derives a key pair for use with the XRP Ledger
    /// from a seed value.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::keypairs::algorithms::Secp256k1;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let decoded_seed: &[u8] = &[
    ///     207, 45, 227, 120, 251, 221, 126, 46,
    ///     232, 125, 72, 109, 251, 90, 123, 255
    /// ];
    /// let validator: bool = false;
    /// let tuple: (String, String) = (
    ///     "0203F2D90BC50012EC7CB20B07A1B818D6863636FB1E945D17449092CFB5495E1E".into(),
    ///     "0048D93A3B5948E5F9B323BF654BFAD6E8FF75B5FCAB03C5A55AD30CB2515B461F".into(),
    /// );
    ///
    /// let derivation: Option<(String, String)> = match Secp256k1.derive_keypair(
    ///     decoded_seed,
    ///     validator,
    /// ) {
    ///     Ok((public, private)) => Some((public, private)),
    ///     Err(e) => match e {
    ///         XRPLKeypairsException::InvalidSignature => None,
    ///         XRPLKeypairsException::InvalidSecret => None,
    ///         XRPLKeypairsException::SECP256K1Error => None,
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
        let (root_public, root_secret) = Self::_derive_part(decoded_seed, Secp256k1Phase::Root)?;
        if is_validator {
            Ok(Secp256k1::_format_keys(root_public, root_secret))
        } else {
            let (mid_public, mid_secret) =
                Self::_derive_part(&root_public.serialize(), Secp256k1Phase::Mid)?;
            let (final_public, final_secret) =
                Self::_derive_final(root_public, root_secret, mid_public, mid_secret)?;

            Ok(Secp256k1::_format_keys(final_public, final_secret))
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
    /// use xrpl::core::keypairs::algorithms::Secp256k1;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let message: &[u8] = "test message".as_bytes();
    /// let private_key: &str = "00D78B9735C3F26501C7337B8A5727FD5\
    ///                          3A6EFDBC6AA55984F098488561F985E23";
    /// let signature: Vec<u8> = vec![
    ///     48, 68, 2, 32, 88, 58, 145, 201, 94, 84, 230, 166, 81, 196,
    ///     123, 236, 34, 116, 78, 11, 16, 30, 44, 64, 96, 231, 176, 143,
    ///     99, 65, 101, 125, 173, 155, 195, 238, 2, 32, 125, 20, 137,
    ///     199, 57, 93, 176, 24, 141, 58, 86, 169, 119, 236, 186, 84,
    ///     179, 111, 169, 55, 27, 64, 49, 150, 85, 177, 180, 66, 158,
    ///     51, 239, 45,
    /// ];
    ///
    /// let signing: Option<Vec<u8>> = match Secp256k1.sign(
    ///     message,
    ///     private_key,
    /// ) {
    ///     Ok(signature) => Some(signature),
    ///     Err(e) => match e {
    ///         XRPLKeypairsException::SECP256K1Error => None,
    ///         _ => None,
    ///     },
    /// };
    ///
    /// assert_eq!(Some(signature), signing);
    /// ```
    fn sign(
        &self,
        message_bytes: &[u8],
        private_key: &str,
    ) -> Result<Vec<u8>, XRPLKeypairsException> {
        let secp = secp256k1::Secp256k1::<secp256k1::SignOnly>::signing_only();
        let message = Self::_get_message(message_bytes)?;
        let trimmed_key = private_key.trim_start_matches(SECP256K1_PREFIX);
        let private = secp256k1::SecretKey::from_str(trimmed_key)?;
        let signature = secp.sign(&message, &private);

        Ok(signature.serialize_der().to_vec())
    }

    /// Verifies the signature on a given message.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::keypairs::algorithms::Secp256k1;
    /// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
    /// use xrpl::core::keypairs::CryptoImplementation;
    ///
    /// let message: &[u8] = "test message".as_bytes();
    /// let signature: &str = "30440220583A91C95E54E6A651C47BEC\
    ///                        22744E0B101E2C4060E7B08F6341657D\
    ///                        AD9BC3EE02207D1489C7395DB0188D3A\
    ///                        56A977ECBA54B36FA9371B40319655B1\
    ///                        B4429E33EF2D";
    /// let public_key: &str = "030D58EB48B4420B1F7B9DF55087E0E\
    ///                         29FEF0E8468F9A6825B01CA2C361042D435";
    ///
    /// assert!(Secp256k1.is_valid_message(
    ///     message,
    ///     signature,
    ///     public_key,
    /// ));
    /// ```
    fn is_valid_message(&self, message_bytes: &[u8], signature: &str, public_key: &str) -> bool {
        let secp = secp256k1::Secp256k1::<secp256k1::VerifyOnly>::verification_only();
        let msg = Self::_get_message(message_bytes);

        if let Ok(value) = hex::decode(signature) {
            let sig = secp256k1::Signature::from_der(&value);
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
    ///         XRPLKeypairsException::InvalidSignature => None,
    ///         XRPLKeypairsException::ED25519Error => None,
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
    /// let signature: Vec<u8> = vec![
    ///     203, 25, 158, 27, 253, 78, 61, 170, 16, 94, 72, 50, 238, 223,
    ///     163, 100, 19, 225, 244, 66, 5,228, 239, 185, 226, 126, 130, 96,
    ///     68, 194, 30, 62, 46, 132, 139, 188, 129, 149, 232, 149, 155,
    ///     173, 248, 135, 89, 155, 115, 16, 173, 27, 112, 71, 239, 17,
    ///     182, 130, 224, 208, 104, 247, 55,73, 117, 14,
    /// ];
    ///
    /// let signing: Option<Vec<u8>> = match Ed25519.sign(
    ///     message,
    ///     private_key,
    /// ) {
    ///     Ok(signature) => Some(signature),
    ///     Err(e) => match e {
    ///         XRPLKeypairsException::ED25519Error => None,
    ///         _ => None,
    ///     },
    /// };
    ///
    /// assert_eq!(Some(signature), signing);
    /// ```
    fn sign(&self, message: &[u8], private_key: &str) -> Result<Vec<u8>, XRPLKeypairsException> {
        let raw_private = hex::decode(&private_key[ED25519_PREFIX.len()..])?;
        let private = ed25519_dalek::SecretKey::from_bytes(&raw_private)?;
        let expanded_private = ed25519_dalek::ExpandedSecretKey::from(&private);
        let public = ed25519_dalek::PublicKey::from(&private);
        let signature: ed25519_dalek::Signature = expanded_private.sign(message, &public);

        Ok(signature.to_bytes().to_vec())
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

        if let (Ok(rpub), Ok(dsig)) = (raw_public, decoded_sig) {
            let public = ed25519_dalek::PublicKey::from_bytes(&rpub);

            if dsig.len() != ED25519_SIGNATURE_LENGTH {
                return false;
            };

            if let Ok(value) = public {
                let sig: [u8; ED25519_SIGNATURE_LENGTH] =
                    dsig.try_into().expect("is_valid_message");
                let converted = &ed25519_dalek::Signature::from(sig);

                value.verify(message, converted).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::test_cases::*;

    #[test]
    fn test_secp256k1_derive_keypair() {
        let seed: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let validator = Secp256k1.derive_keypair(seed, true);
        let (public, private) = Secp256k1.derive_keypair(seed, false).unwrap();

        assert!(validator.is_ok());
        assert_eq!(PRIVATE_SECP256K1, private);
        assert_eq!(PUBLIC_SECP256K1, public);
    }

    #[test]
    fn test_secp256k1_sign() {
        let success = Secp256k1.sign(TEST_MESSAGE.as_bytes(), PRIVATE_SECP256K1);
        let error = Secp256k1.sign(TEST_MESSAGE.as_bytes(), "abc123");

        assert!(success.is_ok());
        assert!(error.is_err());
    }

    #[test]
    fn test_secp256k1_is_valid_message() {
        let signature: &str = &hex::encode_upper(SIGNATURE_SECP256K1);
        let message: &[u8] = TEST_MESSAGE.as_bytes();

        assert!(Secp256k1.is_valid_message(message, signature, PUBLIC_SECP256K1));
    }

    #[test]
    fn test_ed25519_derive_keypair() {
        let seed: &[u8] = SEED_ED25519.as_bytes();
        let validator = Ed25519.derive_keypair(seed, true);
        let (public, private) = Ed25519.derive_keypair(seed, false).unwrap();

        assert!(validator.is_err());
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
