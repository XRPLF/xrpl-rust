//! Miscellaneous helper functions.

use crate::core::types::utils::ACCOUNT_ID_LENGTH;
use core::convert::TryInto;
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

/// Intermediate private keys are always padded with
/// 4 bytes of zeros.
pub(crate) const SECP256K1_INTERMEDIATE_KEYPAIR_PADDING: [u8; 4] = [0, 0, 0, 0];
/// String keys must be _KEY_LENGTH long
pub(crate) const SECP256K1_KEY_LENGTH: usize = 66;

/// Test message for signature verification.
pub(crate) const SIGNATURE_VERIFICATION_MESSAGE: &[u8] = b"This test message should verify.";

/// Length of half a sha512 hash.
pub const SHA512_HASH_LENGTH: usize = 32;

/// ED25519 prefix
pub const ED25519_PREFIX: &str = "ED";

/// Returns the first 32 bytes of SHA-512
/// hash of message.
pub fn sha512_first_half(message: &[u8]) -> [u8; SHA512_HASH_LENGTH] {
    let mut sha512 = Sha512::new();
    sha512.update(message);
    sha512.finalize()[..32]
        .try_into()
        .expect("Invalid slice length")
}

/// Returns the account ID for a given public key.
/// To learn about the relationship between keys
/// and account IDs, see:
/// https://xrpl.org/cryptographic-keys.html#account-id-and-address
pub fn get_account_id(public_key: &[u8]) -> [u8; ACCOUNT_ID_LENGTH] {
    let mut sha256 = Sha256::new();
    let mut ripemd160 = Ripemd160::new();

    sha256.update(public_key);
    ripemd160.update(&sha256.finalize());

    ripemd160.finalize()[..]
        .try_into()
        .expect("Invalid slice length")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::keypairs::test_cases::TEST_ACCOUNT_ID;
    use crate::core::keypairs::test_cases::TEST_MESSAGE;
    use crate::core::keypairs::test_cases::TEST_MESSAGE_SHA;

    #[test]
    fn test_sha512_first_half() {
        assert_eq!(TEST_MESSAGE_SHA, sha512_first_half(TEST_MESSAGE.as_bytes()));
    }

    #[test]
    fn test_get_account_id() {
        assert_eq!(TEST_ACCOUNT_ID, get_account_id(TEST_MESSAGE.as_bytes()));
    }
}
