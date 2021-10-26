//! Core codec functions for interacting with the XRPL.

pub mod algorithms;
pub mod exceptions;
#[cfg(test)]
pub(crate) mod test_cases;
pub mod utils;

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::addresscodec::utils::SEED_LENGTH;
use crate::core::addresscodec::*;
use crate::core::keypairs::algorithms::Ed25519;
use crate::core::keypairs::exceptions::XRPLKeypairsException;
use crate::core::keypairs::utils::*;
use alloc::string::String;
use rand::Rng;
use rand::SeedableRng;

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
fn _get_algorithm_engine(algo: CryptoAlgorithm) -> impl CryptoImplementation {
    match algo {
        CryptoAlgorithm::ED25519 => Ed25519,
        CryptoAlgorithm::SECP256K1 => Ed25519,
    }
}

/// Return the trait implementation based on the
/// provided key.
fn _get_algorithm_engine_from_key(key: &str) -> impl CryptoImplementation {
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
) -> Result<String, XRPLAddressCodecException> {
    let mut random_bytes: [u8; SEED_LENGTH] = [0u8; SEED_LENGTH];
    let algo: CryptoAlgorithm;

    if let Some(value) = algorithm {
        algo = value;
    } else {
        algo = CryptoAlgorithm::ED25519;
    }

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
/// let seed: &str = "sn259rEFXrQrWyx3Q7XneWcwV6dfL";
/// let validator: bool = false;
/// let tuple: (String, String) = (
///     "ED60292139838CB86E719134F848F055057CA5BDA61F5A529729F1697502D53E1C".into(),
///     "ED009F66528611A0D400946A01FA01F8AF4FF4C1D0C744AE3F193317DCA77598F1".into(),
/// );
///
/// let generator: Option<(String, String)> = match derive_keypair(
///     seed,
///     validator,
/// ) {
///     Ok(seed) => Some(seed),
///     Err(e) => match e {
///         XRPLKeypairsException::InvalidSignature => None,
///         XRPLKeypairsException::ED25519Error(_ed25519_error) => None,
///         XRPLKeypairsException::SECP256K1Error(_secp256k1_error) => None,
///         XRPLKeypairsException::UnsupportedValidatorAlgorithm { expected: _ } => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(tuple), generator);
/// ```
pub fn derive_keypair(
    seed: &str,
    validator: bool,
) -> Result<(String, String), XRPLKeypairsException> {
    let (decoded_seed, algorithm) = decode_seed(seed)?;
    let module = _get_algorithm_engine(algorithm);
    let (public, private) = module.derive_keypair(&decoded_seed, validator)?;
    let signature = sign(SIGNATURE_VERIFICATION_MESSAGE, &private)?;

    if module.is_valid_message(SIGNATURE_VERIFICATION_MESSAGE, &signature, &public) {
        Ok((public, private))
    } else {
        Err(XRPLKeypairsException::InvalidSignature)
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
pub fn derive_classic_address(public_key: &str) -> Result<String, XRPLAddressCodecException> {
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
///         XRPLKeypairsException::ED25519Error(_ed25519_error) => None,
///         XRPLKeypairsException::SECP256K1Error(_secp256k1_error) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(signature), signing);
/// ```
pub fn sign(message: &[u8], private_key: &str) -> Result<String, XRPLKeypairsException> {
    let module = _get_algorithm_engine_from_key(private_key);
    module.sign(message, private_key)
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
    ) -> Result<(String, String), XRPLKeypairsException>;

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
    fn sign(&self, message: &[u8], private_key: &str) -> Result<String, XRPLKeypairsException>;

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
    fn is_valid_message(&self, message: &[u8], signature: &str, public_key: &str) -> bool;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::test_cases::*;

    #[test]
    fn test_generate_seed() {
        assert!(generate_seed(None, None).is_ok());
        assert_eq!(SEED_ED25519, generate_seed(Some(TEST_BYTES), None).unwrap());
    }

    #[test]
    fn test_derive_keypair() {
        let (public, private) = derive_keypair(SEED_ED25519, false).unwrap();

        assert_eq!(PRIVATE_ED25519, private);
        assert_eq!(PUBLIC_ED25519, public);
    }

    #[test]
    fn test_derive_classic_address() {
        assert_eq!(
            CLASSIC_ADDRESS_ED25519,
            derive_classic_address(PUBLIC_ED25519).unwrap()
        );
    }

    #[test]
    fn test_sign() {
        assert_eq!(
            hex::encode_upper(SIGNATURE_ED25519),
            sign(TEST_MESSAGE.as_bytes(), PRIVATE_ED25519).unwrap()
        );
    }

    #[test]
    fn test_is_valid_message() {
        let signature: &str = &hex::encode_upper(SIGNATURE_ED25519);
        let message: &[u8] = TEST_MESSAGE.as_bytes();

        assert!(is_valid_message(message, signature, PUBLIC_ED25519));
    }
}
