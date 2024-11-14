//! This module contains commonly-used constants.

pub mod exceptions;
#[cfg(test)]
pub mod test_cases;
pub mod utils;

use crate::constants::CryptoAlgorithm;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::addresscodec::utils::*;
use crate::skip_err;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use strum::IntoEnumIterator;

use super::exceptions::XRPLCoreResult;

/// Map the algorithm to the prefix.
fn _algorithm_to_prefix<'a>(algo: &CryptoAlgorithm) -> &'a [u8] {
    match algo {
        CryptoAlgorithm::ED25519 => &ED25519_SEED_PREFIX,
        CryptoAlgorithm::SECP256K1 => &FAMILY_SEED_PREFIX,
    }
}

/// Returns whether a decoded X-Address is a test address.
fn _is_test_address(prefix: &[u8]) -> XRPLCoreResult<bool> {
    if ADDRESS_PREFIX_BYTES_MAIN == prefix {
        Ok(false)
    } else if ADDRESS_PREFIX_BYTES_TEST == prefix {
        Ok(true)
    } else {
        Err(XRPLAddressCodecException::InvalidXAddressPrefix.into())
    }
}

/// Returns the destination tag extracted from the suffix
/// of the X-Address.
fn _get_tag_from_buffer(buffer: &[u8]) -> XRPLCoreResult<Option<u64>> {
    let flag = &buffer[0];

    if flag >= &2 {
        Err(XRPLAddressCodecException::UnsupportedXAddress.into())
    } else if flag == &1 {
        // Little-endian to big-endian
        Ok(Some(
            buffer[1] as u64
                + buffer[2] as u64 * 0x100
                + buffer[3] as u64 * 0x10000
                + buffer[4] as u64 * 0x1000000,
        ))
        // inverse of what happens in encode
    } else if flag != &0 {
        Err(XRPLAddressCodecException::InvalidXAddressZeroNoTag.into())
    } else if hex::decode("0000000000000000")? != buffer[1..9] {
        Err(XRPLAddressCodecException::InvalidXAddressZeroRemain.into())
    } else {
        Ok(None)
    }
}

/// Returns an encoded seed.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::encode_seed;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::constants::CryptoAlgorithm;
/// use xrpl::core::addresscodec::utils::SEED_LENGTH;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let entropy: [u8; SEED_LENGTH] = [
///     207, 45, 227, 120, 251, 221, 126, 46, 232,
///     125, 72, 109, 251, 90, 123, 255
/// ];
/// let encoding_type: CryptoAlgorithm = CryptoAlgorithm::SECP256K1;
/// let seed: String = "sn259rEFXrQrWyx3Q7XneWcwV6dfL".into();
///
/// let encoding: Option<String> = match encode_seed(
///     entropy,
///     encoding_type,
/// ) {
///     Ok(seed) => Some(seed),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnknownSeedEncoding) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(seed), encoding);
/// ```
pub fn encode_seed(
    entropy: [u8; SEED_LENGTH],
    encoding_type: CryptoAlgorithm,
) -> XRPLCoreResult<String> {
    Ok(encode_base58(
        &entropy,
        _algorithm_to_prefix(&encoding_type),
        Some(SEED_LENGTH),
    )?)
}

/// Returns an encoded seed.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::decode_seed;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::addresscodec::utils::SEED_LENGTH;
/// use xrpl::constants::CryptoAlgorithm;
/// use xrpl::core::exceptions::XRPLCoreException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let seed: &str = "sn259rEFXrQrWyx3Q7XneWcwV6dfL";
/// let tuple: ([u8; SEED_LENGTH], CryptoAlgorithm) = (
///     [207, 45, 227, 120, 251, 221, 126, 46, 232, 125, 72, 109, 251, 90, 123, 255],
///     CryptoAlgorithm::SECP256K1,
/// );
///
/// let decoding: Option<([u8; SEED_LENGTH], CryptoAlgorithm)> = match decode_seed(seed) {
///     Ok((bytes, algorithm)) => Some((bytes, algorithm)),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnknownSeedEncoding) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(tuple), decoding);
/// ```
pub fn decode_seed(seed: &str) -> XRPLCoreResult<([u8; SEED_LENGTH], CryptoAlgorithm)> {
    let mut result: Option<XRPLCoreResult<Vec<u8>>> = None;
    let mut algo: Option<CryptoAlgorithm> = None;

    for a in CryptoAlgorithm::iter() {
        let decode = decode_base58(seed, _algorithm_to_prefix(&a));
        result = Some(skip_err!(decode));
        algo = Some(a);
    }

    match result {
        Some(Ok(val)) => {
            let decoded: [u8; SEED_LENGTH] = val
                .try_into()
                .map_err(|err| XRPLAddressCodecException::VecResizeError(err))?;
            Ok((decoded, algo.expect("decode_seed")))
        }
        Some(Err(_)) | None => Err(XRPLAddressCodecException::UnknownSeedEncoding.into()),
    }
}

/// Returns the X-Address representation of the data.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::classic_address_to_xaddress;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let classic_address: &str = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59".into();
/// let tag: Option<u64> = None;
/// let is_test_network: bool = false;
/// let xaddress: String = "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ".into();
///
/// let conversion: Option<String> = match classic_address_to_xaddress(
///         classic_address,
///         tag,
///         is_test_network
/// ) {
///     Ok(address) => Some(address),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressPrefix) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnsupportedXAddress) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressZeroNoTag) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressZeroRemain) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         }) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(xaddress), conversion);
/// ```
pub fn classic_address_to_xaddress(
    classic_address: &str,
    tag: Option<u64>,
    is_test_network: bool,
) -> XRPLCoreResult<String> {
    let classic_address_bytes = decode_classic_address(classic_address)?;
    let flag: bool = tag.is_some();
    let tag_val: u64;

    if classic_address_bytes.len() != CLASSIC_ADDRESS_ID_LENGTH {
        Err(XRPLAddressCodecException::InvalidCAddressIdLength {
            length: CLASSIC_ADDRESS_ID_LENGTH,
        }
        .into())
    } else if tag.is_some() && tag > Some(u32::MAX.into()) {
        Err(XRPLAddressCodecException::InvalidCAddressTag.into())
    } else {
        if let Some(tval) = tag {
            tag_val = tval;
        } else {
            tag_val = 0;
        }

        let mut bytestring = match is_test_network {
            true => ADDRESS_PREFIX_BYTES_TEST,
            false => ADDRESS_PREFIX_BYTES_MAIN,
        }
        .to_vec();

        bytestring.extend_from_slice(&classic_address_bytes);

        let encoded_tag = [
            flag as u8,
            (tag_val & 0xFF) as u8,
            (tag_val >> 8 & 0xFF) as u8,
            (tag_val >> 16 & 0xFF) as u8,
            (tag_val >> 24 & 0xFF) as u8,
            0,
            0,
            0,
            0,
        ];

        bytestring.extend_from_slice(&encoded_tag);

        Ok(bs58::encode(bytestring)
            .with_alphabet(&XRPL_ALPHABET)
            .with_check()
            .into_string())
    }
}

/// Returns a tuple containing the classic address, tag,
/// and whether the address is on a test network for an
/// X-Address.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::xaddress_to_classic_address;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let xaddress: &str = "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ";
/// let classic: (String, Option<u64>, bool) = (
///     "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59".into(),
///     None,
///     false,
/// );
///
/// let conversion: Option<(String, Option<u64>, bool)> = match xaddress_to_classic_address(xaddress) {
///     Ok((address, tag, is_test_network)) => Some((address, tag, is_test_network)),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressPrefix) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnsupportedXAddress) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressZeroNoTag) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidXAddressZeroRemain) => None,
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         }) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(classic), conversion);
/// ```
pub fn xaddress_to_classic_address(xaddress: &str) -> XRPLCoreResult<(String, Option<u64>, bool)> {
    // Convert b58 to bytes
    let decoded = bs58::decode(xaddress)
        .with_alphabet(&XRPL_ALPHABET)
        .with_check(None)
        .into_vec()?;

    let is_test_network = _is_test_address(&decoded[..2])?;
    let classic_address_bytes = &decoded[2..22];
    // extracts the destination tag
    let tag = _get_tag_from_buffer(&decoded[22..])?;

    let classic_address = encode_classic_address(classic_address_bytes)?;
    Ok((classic_address, tag, is_test_network))
}

/// Returns the classic address encoding of these bytes
/// as a base58 string.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::encode_classic_address;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let bytes: &[u8] = &[
///     94, 123, 17, 37, 35, 246, 141, 47, 94, 135, 157, 180,
///     234, 197, 28, 102, 152, 166, 147, 4
/// ];
/// let address: String = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59".into();
///
/// let encoding: Option<String> = match encode_classic_address(bytes) {
///     Ok(address) => Some(address),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         }) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(address), encoding);
/// ```
pub fn encode_classic_address(bytestring: &[u8]) -> XRPLCoreResult<String> {
    Ok(encode_base58(
        bytestring,
        &CLASSIC_ADDRESS_PREFIX,
        Some(CLASSIC_ADDRESS_LENGTH.into()),
    )?)
}

/// Returns the decoded bytes of the classic address.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::decode_classic_address;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let key: &str = "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59";
/// let bytes: Vec<u8> = vec![
///     94, 123, 17, 37, 35, 246, 141, 47, 94, 135, 157, 180,
///     234, 197, 28, 102, 152, 166, 147, 4
/// ];
///
/// let decoding: Option<Vec<u8>> = match decode_classic_address(key) {
///     Ok(bytes) => Some(bytes),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidEncodingPrefixLength) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(bytes), decoding);
/// ```
pub fn decode_classic_address(classic_address: &str) -> XRPLCoreResult<Vec<u8>> {
    Ok(decode_base58(classic_address, &CLASSIC_ADDRESS_PREFIX)?)
}

/// Returns the node public key encoding of these bytes
/// as a base58 string.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::encode_node_public_key;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let bytes: &[u8] = &[
///     3, 136, 229, 186, 135, 160, 0, 203, 128, 114, 64, 223,
///     140, 132, 142, 176, 181, 255, 165, 200, 229, 165, 33,
///     188, 142, 16, 92, 15, 10, 68, 33, 120, 40
/// ];
/// let key: String = "n9MXXueo837zYH36DvMc13BwHcqtfAWNJY5czWVbp7uYTj7x17TH".into();
///
/// let encoding: Option<String> = match encode_node_public_key(bytes) {
///     Ok(key) => Some(key),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         }) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(key), encoding);
/// ```
pub fn encode_node_public_key(bytestring: &[u8]) -> XRPLCoreResult<String> {
    Ok(encode_base58(
        bytestring,
        &NODE_PUBLIC_KEY_PREFIX,
        Some(NODE_PUBLIC_KEY_LENGTH.into()),
    )?)
}

/// Returns the decoded bytes of the node public key.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::decode_node_public_key;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let key: &str = "n9MXXueo837zYH36DvMc13BwHcqtfAWNJY5czWVbp7uYTj7x17TH";
/// let bytes: Vec<u8> = vec![
///     3, 136, 229, 186, 135, 160, 0, 203, 128, 114, 64, 223,
///     140, 132, 142, 176, 181, 255, 165, 200, 229, 165, 33,
///     188, 142, 16, 92, 15, 10, 68, 33, 120, 40
/// ];
///
/// let decoding: Option<Vec<u8>> = match decode_node_public_key(key) {
///     Ok(bytes) => Some(bytes),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidEncodingPrefixLength) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(bytes), decoding);
/// ```
pub fn decode_node_public_key(node_public_key: &str) -> XRPLCoreResult<Vec<u8>> {
    Ok(decode_base58(node_public_key, &NODE_PUBLIC_KEY_PREFIX)?)
}

/// Returns the account public key encoding of these
/// bytes as a base58 string.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::encode_account_public_key;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
///
/// let bytes: &[u8] = &[
///     2, 54, 147, 241, 89, 103, 174, 53, 125, 3, 39, 151,
///     74, 212, 111, 227, 193, 39, 17, 59, 17, 16, 214, 4,
///     79, 212, 30, 114, 54, 137, 248, 28, 198
/// ];
/// let key: String = "aB44YfzW24VDEJQ2UuLPV2PvqcPCSoLnL7y5M1EzhdW4LnK5xMS3".into();
///
/// let encoding: Option<String> = match encode_account_public_key(bytes) {
///     Ok(key) => Some(key),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::UnexpectedPayloadLength {
///             expected: _,
///             found: _,
///         }) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(key), encoding);
/// ```
pub fn encode_account_public_key(bytestring: &[u8]) -> XRPLCoreResult<String> {
    Ok(encode_base58(
        bytestring,
        &ACCOUNT_PUBLIC_KEY_PREFIX,
        Some(ACCOUNT_PUBLIC_KEY_LENGTH.into()),
    )?)
}

/// Returns the decoded bytes of the node public key.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::decode_account_public_key;
/// use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
/// use xrpl::core::exceptions::XRPLCoreException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let key: &str = "aB44YfzW24VDEJQ2UuLPV2PvqcPCSoLnL7y5M1EzhdW4LnK5xMS3";
/// let bytes: Vec<u8> = vec![
///     2, 54, 147, 241, 89, 103, 174, 53, 125, 3, 39, 151,
///     74, 212, 111, 227, 193, 39, 17, 59, 17, 16, 214, 4,
///     79, 212, 30, 114, 54, 137, 248, 28, 198
/// ];
///
/// let decoding: Option<Vec<u8>> = match decode_account_public_key(key) {
///     Ok(bytes) => Some(bytes),
///     Err(e) => match e {
///         XRPLCoreException::XRPLAddressCodecError(XRPLAddressCodecException::InvalidEncodingPrefixLength) => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(bytes), decoding);
/// ```
pub fn decode_account_public_key(account_public_key: &str) -> XRPLCoreResult<Vec<u8>> {
    Ok(decode_base58(
        account_public_key,
        &ACCOUNT_PUBLIC_KEY_PREFIX,
    )?)
}

/// Returns whether `classic_address` is a valid classic address.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::is_valid_classic_address;
///
/// let address: &str = "rpGaCyHRYbgKhErgFih3RdjJqXDsYBouz3";
///
/// assert!(is_valid_classic_address(address));
/// ```
pub fn is_valid_classic_address(classic_address: &str) -> bool {
    decode_base58(classic_address, &CLASSIC_ADDRESS_PREFIX).is_ok()
}

/// Returns whether ``xaddress`` is a valid X-Address.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::addresscodec::is_valid_xaddress;
///
/// let address: &str = "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ";
///
/// assert!(is_valid_xaddress(address));
/// ```
pub fn is_valid_xaddress(xaddress: &str) -> bool {
    xaddress_to_classic_address(xaddress).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alloc::string::ToString;
    use crate::core::addresscodec::test_cases::*;

    #[test]
    fn test_algorithm_to_prefix() {
        assert_eq!(
            &FAMILY_SEED_PREFIX,
            _algorithm_to_prefix(&CryptoAlgorithm::SECP256K1)
        );
        assert_eq!(
            &ED25519_SEED_PREFIX,
            _algorithm_to_prefix(&CryptoAlgorithm::ED25519)
        );
    }

    #[test]
    fn test_encode_seed() {
        let bytes: [u8; 16] = [
            207, 45, 227, 120, 251, 221, 126, 46, 232, 125, 72, 109, 251, 90, 123, 255,
        ];

        assert_eq!(
            Ok(SECP256K1_ENCODED_SEED_TEST.to_string()),
            encode_seed(bytes, CryptoAlgorithm::SECP256K1)
        );

        let bytes: [u8; 16] = [
            76, 58, 29, 33, 63, 189, 251, 20, 199, 194, 141, 96, 148, 105, 179, 65,
        ];

        assert_eq!(
            Ok(ED25519_ENCODED_SEED_TEST.to_string()),
            encode_seed(bytes, CryptoAlgorithm::ED25519)
        );
    }

    #[test]
    fn test_decode_seed() {
        let hex_bytes = hex::decode(SECP256K1_HEX_TEST).expect("");
        let (decode_result, encoding_type) = decode_seed(SECP256K1_ENCODED_SEED_TEST).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::SECP256K1, encoding_type);

        let hex_bytes = hex::decode(ED25519_HEX_TEST).expect("");
        let (decode_result, encoding_type) = decode_seed(ED25519_ENCODED_SEED_TEST).unwrap();

        assert_eq!(hex_bytes, decode_result);
        assert_eq!(CryptoAlgorithm::ED25519, encoding_type);
    }

    #[test]
    fn test_classic_address_to_xaddress() {
        for case in ADDRESS_TEST_CASES {
            assert_eq!(
                classic_address_to_xaddress(case.address, case.tag, true),
                Ok(case.test_xaddress.to_string()),
            );

            assert_eq!(
                classic_address_to_xaddress(case.address, case.tag, false),
                Ok(case.main_xaddress.to_string()),
            );
        }
    }

    #[test]
    fn test_xaddress_to_classic_address() {
        for case in ADDRESS_TEST_CASES {
            let (classic_address, tag, is_test) =
                xaddress_to_classic_address(case.test_xaddress).unwrap();

            assert_eq!(*case.address, classic_address);
            assert_eq!(case.tag, tag);
            assert!(is_test);

            let (classic_address, tag, is_test) =
                xaddress_to_classic_address(case.main_xaddress).unwrap();

            assert_eq!(*case.address, classic_address);
            assert_eq!(case.tag, tag);
            assert!(!is_test);
        }
    }

    #[test]
    fn test_encode_node_public_key() {
        let bytes = hex::decode(NODE_PUBLIC_KEY_HEX_TEST).expect("");
        assert_eq!(
            encode_node_public_key(&bytes),
            Ok(NODE_PUBLIC_KEY_TEST.to_string()),
        );
    }

    #[test]
    fn test_decode_node_public_key() {
        assert_eq!(
            decode_node_public_key(NODE_PUBLIC_KEY_TEST),
            Ok(hex::decode(NODE_PUBLIC_KEY_HEX_TEST).expect("")),
        );
    }

    #[test]
    fn test_encode_account_public_key() {
        assert_eq!(
            encode_account_public_key(&hex::decode(ACCOUNT_PUBLIC_KEY_HEX_TEST).expect("")),
            Ok(ACCOUNT_PUBLIC_KEY_TEST.to_string()),
        );
    }

    #[test]
    fn test_decode_account_public_key() {
        assert_eq!(
            decode_account_public_key(ACCOUNT_PUBLIC_KEY_TEST),
            Ok(hex::decode(ACCOUNT_PUBLIC_KEY_HEX_TEST).expect("")),
        );
    }

    #[test]
    fn test_is_valid_classic_address() {
        for case in ADDRESS_TEST_CASES {
            assert!(is_valid_classic_address(case.address))
        }
    }

    #[test]
    fn test_is_valid_xaddress() {
        for case in ADDRESS_TEST_CASES {
            assert!(is_valid_xaddress(case.test_xaddress))
        }
    }

    #[test]
    fn accept_seed_encode_decode_secp256k1_low() {
        let encoded_string = "sp6JS7f14BuwFY8Mw6bTtLKWauoUs";
        let bytes: [u8; 16] = Default::default();
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(
            encode_seed(bytes, CryptoAlgorithm::SECP256K1),
            Ok(encoded_string.to_string()),
        );

        assert_eq!(decode_result, bytes);
        assert_eq!(encoding_type, CryptoAlgorithm::SECP256K1);
    }

    #[test]
    fn accept_seed_encode_decode_secp256k1_high() {
        let bytes: [u8; 16] = [255; 16];
        let encoded_string = "saGwBRReqUNKuWNLpUAq8i8NkXEPN";
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(
            encode_seed(bytes, CryptoAlgorithm::SECP256K1),
            Ok(encoded_string.to_string()),
        );

        assert_eq!(decode_result, bytes);
        assert_eq!(encoding_type, CryptoAlgorithm::SECP256K1);
    }

    #[test]
    fn accept_seed_encode_decode_ed25519_low() {
        let encoded_string = "sEdSJHS4oiAdz7w2X2ni1gFiqtbJHqE";
        let bytes: [u8; 16] = Default::default();
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(
            encode_seed(bytes, CryptoAlgorithm::ED25519),
            Ok(encoded_string.to_string()),
        );

        assert_eq!(decode_result, bytes);
        assert_eq!(encoding_type, CryptoAlgorithm::ED25519);
    }

    #[test]
    fn accept_seed_encode_decode_ed25519_high() {
        let bytes: [u8; 16] = [255; 16];
        let encoded_string = "sEdV19BLfeQeKdEXyYA4NhjPJe6XBfG";
        let (decode_result, encoding_type) = decode_seed(encoded_string).unwrap();

        assert_eq!(
            encode_seed(bytes, CryptoAlgorithm::ED25519),
            Ok(encoded_string.to_string()),
        );

        assert_eq!(decode_result, bytes);
        assert_eq!(encoding_type, CryptoAlgorithm::ED25519);
    }
}
