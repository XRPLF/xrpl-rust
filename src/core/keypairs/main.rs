//! Interface for cryptographic key pairs for use
//! with the XRP Ledger.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::codec::decode_seed;
use crate::core::addresscodec::codec::encode_classic_address;
use crate::core::addresscodec::codec::encode_seed;
use crate::core::addresscodec::codec::SEED_LENGTH;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::keypairs::crypto_implementation::CryptoImplementation;
use crate::core::keypairs::ed25519::Ed25519;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::get_account_id;
use alloc::string::String;
use rand::Rng;

/// Test message for signature verification.
const _VERIFICATION_MESSAGE: &[u8] = b"This test message should verify.";

/// Return the trait implementation for the provided
/// algorithm enum.
fn get_algorithm_engine(algo: CryptoAlgorithm) -> impl CryptoImplementation {
    match algo {
        CryptoAlgorithm::ED25519 => Ed25519,
        CryptoAlgorithm::SECP256K1 => Ed25519,
    }
}

/// Generate a seed value that cryptographic keys
/// can be derived from.
pub fn generate_seed(
    entropy: Option<[u8; SEED_LENGTH]>,
    algorithm: Option<CryptoAlgorithm>,
) -> Result<String, XRPLAddressCodecException> {
    let random_bytes: [u8; SEED_LENGTH];
    let algo: CryptoAlgorithm;

    if let Some(value) = algorithm {
        algo = value;
    } else {
        algo = CryptoAlgorithm::ED25519;
    }

    if let Some(value) = entropy {
        random_bytes = value;
    } else {
        random_bytes = rand::thread_rng().gen::<[u8; SEED_LENGTH]>();
    }

    encode_seed(&random_bytes, algo)
}

/// Derive the public and private keys from a given
/// seed value.
pub fn derive_keypair(
    seed: &str,
    validator: bool,
) -> Result<(String, String), XRPLKeypairsException> {
    let (decoded_seed, algorithm) = decode_seed(seed)?;
    let module = get_algorithm_engine(algorithm);
    let (public, private) = module.derive_keypair(&decoded_seed, validator)?;
    let signature = module.sign(_VERIFICATION_MESSAGE, &private)?;

    if module.is_valid_message(_VERIFICATION_MESSAGE, signature, &public) {
        Ok((public, private))
    } else {
        Err(XRPLKeypairsException::new(
            "Derived keypair did not generate verifiable signature",
        ))
    }
}

/// Derive the XRP Ledger classic address for a given
/// public key. For more information, see
/// Address Derivation:
/// https://xrpl.org/cryptographic-keys.html#account-id-and-address
pub fn derive_classic_address(public_key: &str) -> Result<String, XRPLAddressCodecException> {
    let account_id = get_account_id(&hex::decode(public_key)?);
    encode_classic_address(&account_id)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::main_test_cases::BYTES;
    use crate::core::keypairs::main_test_cases::PRIVATE_ED25519;
    use crate::core::keypairs::main_test_cases::PUBLIC_ED25519;
    use crate::core::keypairs::main_test_cases::SEED;

    #[test]
    fn test_generate_seed() {
        assert!(generate_seed(None, None).is_ok());
        assert_eq!(SEED, generate_seed(Some(BYTES), None).unwrap());
    }

    #[test]
    fn test_derive_keypair() {
        let (public, private) = derive_keypair(SEED, false).unwrap();
        alloc::println!("{:?}", public);

        assert_eq!(PRIVATE_ED25519, private);
        assert_eq!(PUBLIC_ED25519, public);
    }

    #[test]
    fn test_derive_classic_address() {
        let expect = "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD";
        assert_eq!(expect, derive_classic_address(PUBLIC_ED25519).unwrap());
    }
}
