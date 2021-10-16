//! This module encodes and decodes various types of base58 encodings.

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::addresscodec::utils::XRPL_ALPHABET;
use crate::skip_err;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use strum::IntoEnumIterator;

/// base58 encodings: https://xrpl.org/base58-encodings.html
/// Account address (20 bytes)
const _CLASSIC_ADDRESS_PREFIX: [u8; 1] = [0x0];
/// value is 35; Account public key (33 bytes)
const _ACCOUNT_PUBLIC_KEY_PREFIX: [u8; 1] = [0x23];
/// value is 33; Seed value (for secret keys) (16 bytes)
const _FAMILY_SEED_PREFIX: [u8; 1] = [0x21];
/// value is 28; Validation public key (33 bytes)
const _NODE_PUBLIC_KEY_PREFIX: [u8; 1] = [0x1C];
/// [1, 225, 75]
const _ED25519_SEED_PREFIX: [u8; 3] = [0x01, 0xE1, 0x4B];

pub const SEED_LENGTH: usize = 16;

const _CLASSIC_ADDRESS_LENGTH: u8 = 20;
const _NODE_PUBLIC_KEY_LENGTH: u8 = 33;
const _ACCOUNT_PUBLIC_KEY_LENGTH: u8 = 33;

/// Map the algorithm to the prefix.
fn algorithm_to_prefix<'a>(algo: &CryptoAlgorithm) -> Option<&'a [u8]> {
    match algo {
        CryptoAlgorithm::ED25519 => Some(&_ED25519_SEED_PREFIX),
        CryptoAlgorithm::SECP256K1 => Some(&_FAMILY_SEED_PREFIX),
    }
}

/// Returns the base58 encoding of the bytestring, with the
/// given data prefix (which indicates type) and while
/// ensuring the bytestring is the expected length.
///
/// See [`bs58::encode`]
///
/// [`bs58::encode`]: bs58::encode
fn _encode(
    bytestring: &[u8],
    prefix: &[u8],
    expected_length: Option<usize>,
) -> Result<String, XRPLAddressCodecException> {
    if expected_length != Some(bytestring.len()) {
        Err(XRPLAddressCodecException::UnexpectedPayloadLength {
            expected: expected_length.unwrap(),
            found: bytestring.len(),
        })
    } else {
        let mut payload = vec![];

        payload.extend_from_slice(prefix);
        payload.extend_from_slice(bytestring);

        Ok(bs58::encode(payload)
            .with_alphabet(&XRPL_ALPHABET)
            .with_check()
            .into_string())
    }
}

/// Returns the byte decoding of the base58-encoded string.
///
/// See [`bs58::encode`]
///
/// [`bs58::encode`]: bs58::encode
fn _decode(b58_string: &str, prefix: &[u8]) -> Result<Vec<u8>, XRPLAddressCodecException> {
    let prefix_len = prefix.len();
    let decoded = bs58::decode(b58_string)
        .with_alphabet(&XRPL_ALPHABET)
        .with_check(None)
        .into_vec()?;

    if &decoded[..prefix_len] != prefix {
        Err(XRPLAddressCodecException::InvalidEncodingPrefixLength)
    } else {
        Ok(decoded[prefix_len..].to_vec())
    }
}

/// Returns an encoded seed.
pub fn encode_seed(
    entropy: &[u8],
    encoding_type: CryptoAlgorithm,
) -> Result<String, XRPLAddressCodecException> {
    let prefix = algorithm_to_prefix(&encoding_type);

    if entropy.len() != SEED_LENGTH {
        Err(XRPLAddressCodecException::InvalidSeedEntropyLength {
            length: SEED_LENGTH,
        })
    } else if prefix == None {
        Err(XRPLAddressCodecException::InvalidSeedPrefixEncodingType)
    } else {
        _encode(entropy, prefix.unwrap(), Some(SEED_LENGTH))
    }
}

/// Returns an encoded seed.
pub fn decode_seed(seed: &str) -> Result<(Vec<u8>, CryptoAlgorithm), XRPLAddressCodecException> {
    let mut result: Option<Result<Vec<u8>, XRPLAddressCodecException>> = None;
    let mut algo: Option<CryptoAlgorithm> = None;

    for a in CryptoAlgorithm::iter() {
        let decode = _decode(seed, algorithm_to_prefix(&a).unwrap());
        result = Some(skip_err!(decode));
        algo = Some(a);
    }

    match result {
        Some(Ok(val)) => Ok((val, algo.unwrap())),
        Some(Err(_)) | None => Err(XRPLAddressCodecException::UnknownSeedEncoding),
    }
}

/// Returns the classic address encoding of these bytes
/// as a base58 string.
pub fn encode_classic_address(bytestring: &[u8]) -> Result<String, XRPLAddressCodecException> {
    _encode(
        bytestring,
        &_CLASSIC_ADDRESS_PREFIX,
        Some(_CLASSIC_ADDRESS_LENGTH.into()),
    )
}

/// Returns the decoded bytes of the classic address.
pub fn decode_classic_address(classic_address: &str) -> Result<Vec<u8>, XRPLAddressCodecException> {
    _decode(classic_address, &_CLASSIC_ADDRESS_PREFIX)
}

/// Returns the node public key encoding of these bytes
/// as a base58 string.
pub fn encode_node_public_key(bytestring: &[u8]) -> Result<String, XRPLAddressCodecException> {
    _encode(
        bytestring,
        &_NODE_PUBLIC_KEY_PREFIX,
        Some(_NODE_PUBLIC_KEY_LENGTH.into()),
    )
}

/// Returns the decoded bytes of the node public key.
pub fn decode_node_public_key(node_public_key: &str) -> Result<Vec<u8>, XRPLAddressCodecException> {
    _decode(node_public_key, &_NODE_PUBLIC_KEY_PREFIX)
}

/// Returns the account public key encoding of these
/// bytes as a base58 string.
pub fn encode_account_public_key(bytestring: &[u8]) -> Result<String, XRPLAddressCodecException> {
    _encode(
        bytestring,
        &_ACCOUNT_PUBLIC_KEY_PREFIX,
        Some(_ACCOUNT_PUBLIC_KEY_LENGTH.into()),
    )
}

/// Returns the decoded bytes of the node public key.
pub fn decode_account_public_key(
    account_public_key: &str,
) -> Result<Vec<u8>, XRPLAddressCodecException> {
    _decode(account_public_key, &_ACCOUNT_PUBLIC_KEY_PREFIX)
}

/// Returns whether `classic_address` is a valid classic address.
pub fn is_valid_classic_address(classic_address: &str) -> bool {
    decode_classic_address(classic_address).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_algorithm_to_prefix() {
        assert_eq!(
            &_FAMILY_SEED_PREFIX,
            algorithm_to_prefix(&CryptoAlgorithm::SECP256K1).unwrap()
        );
        assert_eq!(
            &_ED25519_SEED_PREFIX,
            algorithm_to_prefix(&CryptoAlgorithm::ED25519).unwrap()
        );
    }

    #[test]
    fn test_encode_seed() {
        let encoded_string = "sn259rEFXrQrWyx3Q7XneWcwV6dfL";
        let hex_bytes = hex::decode("CF2DE378FBDD7E2EE87D486DFB5A7BFF").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::SECP256K1);

        assert_eq!(encoded_string, encode_result.unwrap());

        let encoded_string = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let hex_bytes = hex::decode("4C3A1D213FBDFB14C7C28D609469B341").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::ED25519);

        assert_eq!(encoded_string, encode_result.unwrap());
    }

    #[test]
    fn test_decode_seed() {
        let encoded_string = "sn259rEFXrQrWyx3Q7XneWcwV6dfL";
        let hex_bytes = hex::decode("CF2DE378FBDD7E2EE87D486DFB5A7BFF").unwrap();
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::SECP256K1, encoding_type);

        let encoded_string = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let hex_bytes = hex::decode("4C3A1D213FBDFB14C7C28D609469B341").unwrap();
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::ED25519, encoding_type);
    }

    #[test]
    fn accept_seed_encode_decode_secp256k1_low() {
        let encoded_string = "sp6JS7f14BuwFY8Mw6bTtLKWauoUs";
        let hex_bytes = hex::decode("00000000000000000000000000000000").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::SECP256K1);

        assert_eq!(encoded_string, encode_result.unwrap());

        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::SECP256K1, encoding_type);
    }

    #[test]
    fn accept_seed_encode_decode_secp256k1_high() {
        let hex_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        let encoded_string = "saGwBRReqUNKuWNLpUAq8i8NkXEPN";
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::SECP256K1);

        assert_eq!(encoded_string, encode_result.unwrap());

        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::SECP256K1, encoding_type);
    }

    #[test]
    fn accept_seed_encode_decode_ed25519_low() {
        let encoded_string = "sEdSJHS4oiAdz7w2X2ni1gFiqtbJHqE";
        let hex_bytes = hex::decode("00000000000000000000000000000000").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::ED25519);

        assert_eq!(encoded_string, encode_result.unwrap());

        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::ED25519, encoding_type);
    }

    #[test]
    fn accept_seed_encode_decode_ed25519_high() {
        let hex_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
        let encoded_string = "sEdV19BLfeQeKdEXyYA4NhjPJe6XBfG";
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::ED25519);

        assert_eq!(encoded_string, encode_result.unwrap());

        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::ED25519, encoding_type);
    }

    #[test]
    fn accept_seed_encode_decode_too_small() {
        let hex_bytes = hex::decode("CF2DE378FBDD7E2EE87D486DFB5A7B").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::SECP256K1);

        assert!(encode_result.is_err());
    }

    #[test]
    fn accept_seed_encode_decode_too_big() {
        let hex_bytes = hex::decode("CF2DE378FBDD7E2EE87D486DFB5A7BFFFF").unwrap();
        let encode_result = encode_seed(&hex_bytes, CryptoAlgorithm::SECP256K1);

        assert!(encode_result.is_err());
    }
}
