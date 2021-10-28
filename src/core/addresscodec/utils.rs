//! This module contains commonly-used utilities.

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bs58::Alphabet;

/// value is 33; Seed value (for secret keys) (16 bytes)
pub(crate) const FAMILY_SEED_PREFIX: [u8; 1] = [0x21];
/// [1, 225, 75]
pub(crate) const ED25519_SEED_PREFIX: [u8; 3] = [0x01, 0xE1, 0x4B];

/// Bytes for the prefix of a mainnet address.
pub(crate) const ADDRESS_PREFIX_BYTES_MAIN: [u8; 2] = [0x05, 0x44];
/// Bytes for the prefix of a testnet address.
pub(crate) const ADDRESS_PREFIX_BYTES_TEST: [u8; 2] = [0x04, 0x93];

/// Length of a classic address.
pub(crate) const CLASSIC_ADDRESS_ID_LENGTH: usize = 20;
/// Classic address length
pub(crate) const CLASSIC_ADDRESS_LENGTH: u8 = 20;
/// base58 encodings: https://xrpl.org/base58-encodings.html
/// Account address (20 bytes)
pub(crate) const CLASSIC_ADDRESS_PREFIX: [u8; 1] = [0x0];

/// value is 28; Validation public key (33 bytes)
pub(crate) const NODE_PUBLIC_KEY_PREFIX: [u8; 1] = [0x1C];
/// Node public key length.
pub(crate) const NODE_PUBLIC_KEY_LENGTH: u8 = 33;
/// Value is 35; Account public key (33 bytes)
pub(crate) const ACCOUNT_PUBLIC_KEY_PREFIX: [u8; 1] = [0x23];
/// Account public key length
pub(crate) const ACCOUNT_PUBLIC_KEY_LENGTH: u8 = 33;

/// The dictionary used for XRPL base58 encodings
/// Sourced from the [`bs58`] crate.
///
/// [`bs58`]: bs58::Alphabet
pub const XRPL_ALPHABET: Alphabet = *bs58::Alphabet::RIPPLE;

/// Lenght of a seed value.
pub const SEED_LENGTH: usize = 16;

/// Returns the byte decoding of the base58-encoded string.
///
/// See [`bs58::encode`]
///
/// [`bs58::encode`]: mod@bs58::encode()
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::utils::decode_base58;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let encoded: &str = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59";
/// let prefix: &[u8] = &[0x0];
/// let decoded: Vec<u8> = vec![
///     94, 123, 17, 37, 35, 246, 141, 47, 94, 135, 157, 180,
///     234, 197, 28, 102, 152, 166, 147, 4,
/// ];
///
/// let result: Option<Vec<u8>> = match decode_base58(
///     encoded,
///     prefix
/// ) {
///     Ok(val) => Some(val),
///     Err(e) => match e {
///         XRPLAddressCodecException::InvalidEncodingPrefixLength => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(decoded), result);
/// ```
pub fn decode_base58(
    b58_string: &str,
    prefix: &[u8],
) -> Result<Vec<u8>, XRPLAddressCodecException> {
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

/// Returns the base58 encoding of the bytestring, with the
/// given data prefix (which indicates type) and while
/// ensuring the bytestring is the expected length.
///
/// See [`bs58::encode`]
///
/// [`bs58::encode`]: mod@bs58::encode()
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::utils::encode_base58;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
///
/// let decoded: &[u8] = &[
///     94, 123, 17, 37, 35, 246, 141, 47, 94, 135, 157, 180,
///     234, 197, 28, 102, 152, 166, 147, 4,
/// ];
/// let prefix: &[u8] = &[0x0];
/// let expected_length: Option<usize> = Some(20);
/// let encoded: String = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59".to_string();
///
/// let result: Option<String> = match encode_base58(
///     decoded,
///     prefix,
///     expected_length
/// ) {
///     Ok(val) => Some(val),
///     Err(e) => match e {
///         XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         } => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(encoded), result);
/// ```
pub fn encode_base58(
    bytestring: &[u8],
    prefix: &[u8],
    expected_length: Option<usize>,
) -> Result<String, XRPLAddressCodecException> {
    if expected_length != Some(bytestring.len()) {
        Err(XRPLAddressCodecException::UnexpectedPayloadLength {
            expected: expected_length.expect(""),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;

    const ENCODED: &str = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59";
    const DECODED: &[u8] = &[
        94, 123, 17, 37, 35, 246, 141, 47, 94, 135, 157, 180, 234, 197, 28, 102, 152, 166, 147, 4,
    ];

    #[test]
    fn test_decode_base58() {
        assert_eq!(decode_base58(ENCODED, &[0x0]), Ok(DECODED.to_vec()));
    }

    #[test]
    fn test_encode_base58() {
        assert_eq!(
            encode_base58(DECODED, &[0x0], Some(20)),
            Ok(ENCODED.to_string())
        );
    }
}
