//! Codec for currency property inside an XRPL
//! issued currency amount json.

use crate::constants::HEX_CURRENCY_REGEX;
use crate::constants::ISO_CURRENCY_REGEX;
use crate::utils::exceptions::ISOCodeException;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;
use regex::Regex;

const _CURRENCY_CODE_LENGTH: usize = 20;

/// Tests if value is a valid 3-char iso code.
fn _is_iso_code(value: &str) -> bool {
    let regex = Regex::new(ISO_CURRENCY_REGEX).unwrap();
    regex.is_match(value)
}

/// Tests if value is a valid 40-char hex string.
fn _is_hex(value: &str) -> bool {
    let regex = Regex::new(HEX_CURRENCY_REGEX).unwrap();
    regex.is_match(value)
}

fn _iso_code_from_hex(value: &[u8]) -> Result<Option<String>, ISOCodeException> {
    let candidate_iso = hex::encode(value);

    if candidate_iso == "XRP" {
        Err(ISOCodeException::InvalidXRPBytes)
    } else if _is_iso_code(&candidate_iso) {
        Ok(Some(candidate_iso))
    } else {
        Ok(None)
    }
}

/// Convert an ISO code to a 160-bit (20 byte) encoded
/// representation.
///
/// See "Currency codes" subheading in Amount Fields:
/// `<https://xrpl.org/serialization.html#amount-fields>`
fn _iso_to_bytes(value: &str) -> Result<[u8; _CURRENCY_CODE_LENGTH], ISOCodeException> {
    if _is_iso_code(value) {
        Err(ISOCodeException::InvalidISOCode)
    } else if value == "XRP" {
        Ok([0; _CURRENCY_CODE_LENGTH])
    } else {
        let pad_left: [u8; 12] = [0; 12];
        let pad_right: [u8; 5] = [0; 5];
        let iso_bytes = hex::decode(value)?;
        let mut result: Vec<u8> = vec![];

        result.extend_from_slice(&pad_left);
        result.extend_from_slice(&iso_bytes);
        result.extend_from_slice(&pad_right);
        Ok(result
            .try_into()
            .or(Err(ISOCodeException::InvalidISOLength))?)
    }
}
