//! Miscellaneous helper functions.

use core::convert::TryInto;
use ripemd160::Ripemd160;
use sha2::{Digest, Sha512};

/// Returns the first 32 bytes of SHA-512
/// hash of message.
pub fn sha512_first_half(message: &[u8]) -> [u8; 32] {
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
pub fn get_account_id(public_key: &[u8]) -> [u8; 20] {
    let mut sha512 = Sha512::new();
    let mut ripemd160 = Ripemd160::new();

    sha512.update(public_key);
    ripemd160.update(&sha512.finalize());

    ripemd160.finalize()[..]
        .try_into()
        .expect("Invalid slice length")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sha512_first_half() {
        let expect: [u8; 32] = [
            134, 24, 68, 214, 112, 78, 133, 115, 254, 195, 77, 150, 126, 32, 188, 254, 243, 212,
            36, 207, 72, 190, 4, 230, 220, 8, 242, 189, 88, 199, 41, 116,
        ];
        assert_eq!(expect, sha512_first_half(b"Hello World!"));
    }

    #[test]
    fn test_get_account_id() {
        let expect: [u8; 20] = [
            241, 172, 163, 132, 33, 73, 15, 228, 92, 247, 204, 190, 200, 168, 106, 184, 215, 244,
            32, 41,
        ];
        assert_eq!(expect, get_account_id(b"Hello World!"));
    }
}
