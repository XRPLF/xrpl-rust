//! Miscellaneous helper functions.

use crate::constants::ACCOUNT_ID_LENGTH;
use core::convert::TryInto;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

/// Intermediate private keys are always padded with
/// 4 bytes of zeros.
pub(crate) const SECP256K1_INTERMEDIATE_KEYPAIR_PADDING: [u8; 4] = [0, 0, 0, 0];
/// String keys must be _KEY_LENGTH long
pub(crate) const SECP256K1_KEY_LENGTH: usize = 66;
/// SECP256K1 sequence size.
pub(crate) const SECP256K1_SEQUENCE_SIZE: u32 = 4;
/// SECP256K1 maximum sequence.
pub(crate) const SECP256K1_SEQUENCE_MAX: u64 = u64::pow(256, SECP256K1_SEQUENCE_SIZE);

/// Test message for signature verification.
pub(crate) const SIGNATURE_VERIFICATION_MESSAGE: &[u8] = b"This test message should verify.";

/// String keys must be _KEY_LENGTH long
pub const SECP256K1_SIGNATURE_LENGTH: usize = secp256k1::constants::MAX_SIGNATURE_SIZE;
/// String keys must be _KEY_LENGTH long
pub const ED25519_SIGNATURE_LENGTH: usize = ed25519_dalek::SIGNATURE_LENGTH;
/// Length of half a sha512 hash.
pub const SHA512_HASH_LENGTH: usize = 32;

/// ED25519 prefix
pub const ED25519_PREFIX: &str = "ED";
/// SECP256K1 prefix
pub const SECP256K1_PREFIX: char = '0';

#[derive(Debug, PartialEq)]
pub(crate) enum Secp256k1Phase {
    Root,
    Mid,
}

/// Returns the first 32 bytes of SHA-512 hash of message.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::utils::sha512_first_half;
/// use xrpl::core::keypairs::utils::SHA512_HASH_LENGTH;
///
/// let message: &[u8] = "test message".as_bytes();
/// let sha_result: [u8; SHA512_HASH_LENGTH] = [
///     149, 11, 42, 126, 255, 167, 143, 81, 166, 53, 21,
///     236, 69, 224, 62, 206, 190, 80, 239, 47, 28, 65,
///     230, 150, 41, 181, 7, 120, 241, 27, 192, 128,
/// ];
///
/// assert_eq!(sha_result, sha512_first_half(message));
/// ```
pub fn sha512_first_half(message: &[u8]) -> [u8; SHA512_HASH_LENGTH] {
    let mut sha512 = Sha512::new();

    sha512.update(message);
    sha512.finalize()[..SHA512_HASH_LENGTH]
        .try_into()
        .expect("Invalid slice length")
}

/// Returns the account ID for a given public key.
///
/// See Account ID and Address:
/// `<https://xrpl.org/cryptographic-keys.html#account-id-and-address>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::keypairs::utils::get_account_id;
/// use xrpl::constants::ACCOUNT_ID_LENGTH;
///
/// let public_key: &[u8] = "test message".as_bytes();
/// let account_id: [u8; ACCOUNT_ID_LENGTH] = [
///     159, 127, 79, 53, 164, 208, 121, 119, 65, 32,
///     123, 166, 113, 156, 97, 173, 156, 182, 36, 249,
/// ];
///
/// assert_eq!(account_id, get_account_id(public_key));
/// ```
pub fn get_account_id(public_key: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
    let mut sha256 = Sha256::new();
    let mut ripemd160 = Ripemd160::new();

    sha256.update(public_key);
    ripemd160.update(sha256.finalize());

    ripemd160.finalize()[..ACCOUNT_ID_LENGTH]
        .try_into()
        .expect("Invalid slice length")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::test_cases::*;

    #[test]
    fn test_sha512_first_half() {
        assert_eq!(TEST_MESSAGE_SHA, sha512_first_half(TEST_MESSAGE.as_bytes()));
    }

    #[test]
    fn test_get_account_id() {
        assert_eq!(TEST_ACCOUNT_ID, get_account_id(TEST_MESSAGE.as_bytes()));
    }
}
