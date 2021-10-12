//! This module handles everything related to X-Addresses.
//! To better understand the cryptographic details, visit
//! https://github.com/xrp-community/standards-drafts/issues/6
//!
//! General format of an X-Address:
//! [← 2 byte prefix →|← 160 bits of account ID →|← 8 bits of flags →|← 64 bits of tag →]

use crate::core::addresscodec::codec::decode_classic_address;
use crate::core::addresscodec::codec::encode_classic_address;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::addresscodec::utils::XRPL_ALPHABET;
use alloc::string::String;

const MAX_32_BIT_UNSIGNED_INT: u32 = u32::max_value();

const _PREFIX_BYTES_MAIN: [u8; 2] = [0x05, 0x44];
const _PREFIX_BYTES_TEST: [u8; 2] = [0x04, 0x93];

/// Returns whether a decoded X-Address is a test address.
fn _is_test_address(prefix: &[u8]) -> Result<bool, XRPLAddressCodecException> {
    if _PREFIX_BYTES_MAIN == prefix {
        Ok(false)
    } else if _PREFIX_BYTES_TEST == prefix {
        Ok(true)
    } else {
        Err(XRPLAddressCodecException::new(
            "Invalid X-Address: bad prefix",
        ))
    }
}

/// Returns the destination tag extracted from the suffix
/// of the X-Address.
fn _get_tag_from_buffer(buffer: &[u8]) -> Result<Option<u64>, XRPLAddressCodecException> {
    let flag = &buffer[0];

    if flag >= &2 {
        Err(XRPLAddressCodecException::new("Unsupported X-Address"))
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
        Err(XRPLAddressCodecException::new(
            "Flag must be zero to indicate no tag",
        ))
    } else if hex::decode("0000000000000000")? != buffer[1..9] {
        Err(XRPLAddressCodecException::new(
            "Remaining bytes must be zero",
        ))
    } else {
        Ok(None)
    }
}

/// Returns whether ``xaddress`` is a valid X-Address.
pub fn is_valid_xaddress(xaddress: &str) -> bool {
    xaddress_to_classic_address(xaddress).is_ok()
}

/// Returns the X-Address representation of the data.
pub fn classic_address_to_xaddress(
    classic_address: &str,
    tag: Option<u64>,
    is_test_network: bool,
) -> Result<String, XRPLAddressCodecException> {
    let classic_address_bytes = decode_classic_address(classic_address)?;
    let flag: bool = tag != None;
    let tag_val: u64;

    if classic_address_bytes.len() != 20 {
        Err(XRPLAddressCodecException::new(
            "Account ID must be 20 bytes",
        ))
    } else if tag != None && tag > Some(MAX_32_BIT_UNSIGNED_INT.into()) {
        Err(XRPLAddressCodecException::new("Invalid tag"))
    } else {
        if tag != None {
            tag_val = tag.unwrap();
        } else {
            tag_val = 0;
        }

        let mut bytestring = match is_test_network {
            true => _PREFIX_BYTES_TEST,
            false => _PREFIX_BYTES_MAIN,
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
pub fn xaddress_to_classic_address(
    xaddress: &str,
) -> Result<(String, Option<u64>, bool), XRPLAddressCodecException> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::addresscodec::test_cases::ADDRESS_TEST_CASES;

    #[test]
    fn test_is_valid_xaddress() {
        for case in ADDRESS_TEST_CASES {
            assert!(is_valid_xaddress(case.test_xaddress))
        }
    }

    #[test]
    fn test_classic_address_to_xaddress() {
        for case in ADDRESS_TEST_CASES {
            let xtest = classic_address_to_xaddress(case.address, case.tag, true);
            assert_eq!(*case.test_xaddress, xtest.unwrap());

            let xmain = classic_address_to_xaddress(case.address, case.tag, false);
            assert_eq!(*case.main_xaddress, xmain.unwrap());
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
}
