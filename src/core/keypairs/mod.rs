//! Core codec functions for interacting with the XRPL.

pub mod algorithms;
pub mod exceptions;
#[cfg(test)]
pub(crate) mod test_cases;
pub mod utils;

pub use self::algorithms::Ed25519;
pub use self::algorithms::Secp256k1;

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::utils::SEED_LENGTH;
use crate::core::addresscodec::*;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rand::Rng;
use rand::SeedableRng;

use super::exceptions::XRPLCoreResult;

/// Return the signature length for an algorithm.
const fn _get_algorithm_sig_length(algo: CryptoAlgorithm) -> usize {
    match algo {
        CryptoAlgorithm::ED25519 => ED25519_SIGNATURE_LENGTH,
        CryptoAlgorithm::SECP256K1 => SECP256K1_SIGNATURE_LENGTH,
    }
}

/// Return the CryptoAlgorithm from a key.
fn _get_algorithm_from_key(key: &str) -> CryptoAlgorithm {
    match &key[..2] {
        ED25519_PREFIX => CryptoAlgorithm::ED25519,
        _ => CryptoAlgorithm::SECP256K1,
    }
}

/// Return the trait implementation for the provided
/// algorithm enum.
fn _get_algorithm_engine(algo: CryptoAlgorithm) -> Box<dyn CryptoImplementation> {
    match algo {
        CryptoAlgorithm::ED25519 => Box::new(Ed25519),
        CryptoAlgorithm::SECP256K1 => Box::new(Secp256k1),
    }
}

/// Return the trait implementation based on the
/// provided key.
fn _get_algorithm_engine_from_key(key: &str) -> Box<dyn CryptoImplementation> {
    match &key[..2] {
        ED25519_PREFIX => _get_algorithm_engine(CryptoAlgorithm::ED25519),
        _ => _get_algorithm_engine(CryptoAlgorithm::SECP256K1),
    }
}

/// Generate a seed value that cryptographic keys
/// can be derived from.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::generate_seed;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::constants::CryptoAlgorithm;
/// use xrpl::core::addresscodec::utils::SEED_LENGTH;
///
/// let entropy: Option<[u8; SEED_LENGTH]> = Some([
///     207, 45, 227, 120, 251, 221, 126, 46,
///     232, 125, 72, 109, 251, 90, 123, 255
/// ]);
/// let algorithm: Option<CryptoAlgorithm> = Some(CryptoAlgorithm::SECP256K1);
/// let seed: String = "sn259rEFXrQrWyx3Q7XneWcwV6dfL".into();
///
/// let generator: Option<String> = match generate_seed(
///     entropy,
///     algorithm,
/// ) {
///     Ok(seed) => Some(seed),
///     Err(e) => match e {
///         XRPLAddressCodecException::UnknownSeedEncoding => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(seed), generator);
/// ```
pub fn generate_seed(
    entropy: Option<[u8; SEED_LENGTH]>,
    algorithm: Option<CryptoAlgorithm>,
) -> XRPLCoreResult<String> {
    let mut random_bytes: [u8; SEED_LENGTH] = [0u8; SEED_LENGTH];

    let algo: CryptoAlgorithm = if let Some(value) = algorithm {
        value
    } else {
        CryptoAlgorithm::ED25519
    };

    if let Some(value) = entropy {
        random_bytes = value;
    } else {
        let mut rng = rand_hc::Hc128Rng::from_entropy();
        rng.fill(&mut random_bytes);
    }

    encode_seed(random_bytes, algo)
}

/// Derive the public and private keys from a given seed value.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::derive_keypair;
/// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
///
/// let seed: &str = "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r";
/// let validator: bool = false;
/// let tuple: (String, String) = (
///     "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63".into(),
///     "EDB4C4E046826BD26190D09715FC31F4E6A728204EADD112905B08B14B7F15C4F3".into(),
/// );
///
/// let generator: Option<(String, String)> = match derive_keypair(
///     seed,
///     validator,
/// ) {
///     Ok(seed) => Some(seed),
///     Err(e) => match e {
///         XRPLKeypairsException::InvalidSignature => None,
///         XRPLKeypairsException::ED25519Error => None,
///         XRPLKeypairsException::SECP256K1Error => None,
///         XRPLKeypairsException::UnsupportedValidatorAlgorithm { expected: _ } => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(tuple), generator);
/// ```
pub fn derive_keypair(seed: &str, validator: bool) -> XRPLCoreResult<(String, String)> {
    let (decoded_seed, algorithm) = decode_seed(seed)?;
    let module = _get_algorithm_engine(algorithm);
    let (public, private) = module.derive_keypair(&decoded_seed, validator)?;
    let signature = sign(SIGNATURE_VERIFICATION_MESSAGE, &private)?;

    if module.is_valid_message(SIGNATURE_VERIFICATION_MESSAGE, &signature, &public) {
        Ok((public, private))
    } else {
        Err(XRPLKeypairsException::InvalidSignature.into())
    }
}

/// Derive the XRP Ledger classic address for a given
/// public key. For more information, see Address Derivation.
///
/// Account ID and Address:
/// `<https://xrpl.org/cryptographic-keys.html#account-id-and-address>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::derive_classic_address;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
///
/// let public_key: &str = "ED01FA53FA5A7E77798F882ECE20B1ABC00\
///                         BB358A9E55A202D0D0676BD0CE37A63";
/// let address: String = "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD".into();
///
/// let derivation: Option<String> = match derive_classic_address(public_key) {
///     Ok(address) => Some(address),
///     Err(e) => match e {
///         XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         } => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(address), derivation);
/// ```
pub fn derive_classic_address(public_key: &str) -> XRPLCoreResult<String> {
    let account_id = get_account_id(&hex::decode(public_key)?);
    encode_classic_address(&account_id)
}

/// Sign a message using a given private key.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::sign;
/// use xrpl::core::keypairs::exceptions::XRPLKeypairsException;
///
/// let message: &[u8] = "test message".as_bytes();
/// let private_key: &str = "EDB4C4E046826BD26190D09715FC31F4E\
///                          6A728204EADD112905B08B14B7F15C4F3";
/// let signature: String = "CB199E1BFD4E3DAA105E4832EEDFA36413E1F44205E4EFB9\
///                          E27E826044C21E3E2E848BBC8195E8959BADF887599B7310\
///                          AD1B7047EF11B682E0D068F73749750E".into();
///
/// let signing: Option<String> = match sign(
///     message,
///     private_key,
/// ) {
///     Ok(signature) => Some(signature),
///     Err(e) => match e {
///         XRPLKeypairsException::ED25519Error => None,
///         XRPLKeypairsException::SECP256K1Error => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(signature), signing);
/// ```
pub fn sign(message: &[u8], private_key: &str) -> XRPLCoreResult<String> {
    let module = _get_algorithm_engine_from_key(private_key);
    Ok(hex::encode_upper(module.sign(message, private_key)?))
}

/// Verifies the signature on a given message.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::is_valid_message;
///
/// let message: &[u8] = "test message".as_bytes();
/// let signature: &str = "CB199E1BFD4E3DAA105E4832EEDFA36413E1F44205E4EFB9\
///                        E27E826044C21E3E2E848BBC8195E8959BADF887599B7310\
///                        AD1B7047EF11B682E0D068F73749750E";
/// let public_key: &str = "ED01FA53FA5A7E77798F882ECE20B1ABC00\
///                         BB358A9E55A202D0D0676BD0CE37A63";
///
/// assert!(is_valid_message(
///     message,
///     signature,
///     public_key,
/// ));
/// ```
pub fn is_valid_message(message: &[u8], signature: &str, public_key: &str) -> bool {
    let module = _get_algorithm_engine_from_key(public_key);
    module.is_valid_message(message, signature, public_key)
}

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
    ) -> XRPLCoreResult<(String, String)>;

    /// Signs a message using a given private key.
    /// * `message` - Text about foo.
    /// * `private_key` - Text about bar.
    fn sign(&self, message: &[u8], private_key: &str) -> XRPLCoreResult<Vec<u8>>;

    /// Verifies the signature on a given message.
    fn is_valid_message(&self, message: &[u8], signature: &str, public_key: &str) -> bool;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;
    use crate::constants::CryptoAlgorithm;
    use crate::core::keypairs::test_cases::*;

    #[test]
    fn test_generate_seed() {
        assert!(generate_seed(None, None).is_ok());
        assert!(generate_seed(Some(TEST_BYTES), None).is_ok());
        assert_eq!(
            generate_seed(Some(TEST_BYTES), Some(CryptoAlgorithm::ED25519)),
            Ok(SEED_ED25519.to_string()),
        );
        assert_eq!(
            generate_seed(Some(TEST_BYTES), Some(CryptoAlgorithm::SECP256K1)),
            Ok(SEED_SECP256K1.to_string()),
        );
    }

    #[test]
    fn test_derive_keypair() {
        let (public_ed25519, private_ed25519) = derive_keypair(SEED_ED25519, false).unwrap();
        let (public_secp256k1, private_secp256k1) = derive_keypair(SEED_SECP256K1, false).unwrap();

        assert_eq!(PRIVATE_ED25519, private_ed25519);
        assert_eq!(PUBLIC_ED25519, public_ed25519);
        assert_eq!(PRIVATE_SECP256K1, private_secp256k1);
        assert_eq!(PUBLIC_SECP256K1, public_secp256k1);
    }

    #[test]
    fn test_derive_classic_address() {
        assert_eq!(
            derive_classic_address(PUBLIC_ED25519),
            Ok(CLASSIC_ADDRESS_ED25519.to_string()),
        );

        assert_eq!(
            derive_classic_address(PUBLIC_SECP256K1),
            Ok(CLASSIC_ADDRESS_SECP256K1.to_string()),
        );
    }

    #[test]
    fn test_sign() {
        assert_eq!(
            sign(TEST_MESSAGE.as_bytes(), PRIVATE_ED25519),
            Ok(hex::encode_upper(SIGNATURE_ED25519)),
        );

        assert_eq!(
            sign(TEST_MESSAGE.as_bytes(), PRIVATE_SECP256K1),
            Ok(hex::encode_upper(SIGNATURE_SECP256K1)),
        );
    }

    #[test]
    fn test_is_valid_message() {
        let message: &[u8] = TEST_MESSAGE.as_bytes();
        let sig_ed25519: &str = &hex::encode_upper(SIGNATURE_ED25519);
        let sig_secp256k1: &str = &hex::encode_upper(SIGNATURE_SECP256K1);

        assert!(is_valid_message(message, sig_ed25519, PUBLIC_ED25519));
        assert!(is_valid_message(message, sig_secp256k1, PUBLIC_SECP256K1));
    }
}
