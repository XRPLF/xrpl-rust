//! Miscellaneous helper functions.

use crate::core::binarycodec::types::utils::ACCOUNT_ID_LENGTH;
use core::convert::TryInto;
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

/// Length of half a sha512 hash.
pub const HASH_LENGTH: usize = 32;

/// Returns the first 32 bytes of SHA-512
/// hash of message.
pub fn sha512_first_half(message: &[u8]) -> [u8; HASH_LENGTH] {
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
