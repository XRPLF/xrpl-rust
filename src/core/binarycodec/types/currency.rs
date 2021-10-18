//! Codec for currency property inside an XRPL
//! issued currency amount json.

use crate::constants::HEX_CURRENCY_REGEX;
use crate::constants::ISO_CURRENCY_REGEX;
use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::types::hash160::Hash160;
use crate::core::binarycodec::types::serialized_type::Buffered;
use crate::core::binarycodec::types::serialized_type::Serializable;
use crate::utils::exceptions::ISOCodeException;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const CURRENCY_CODE_LENGTH: usize = 20;
pub const NATIVE_HEX_CODE: &str = "0000000000000000000000000000000000000000";
pub const NATIVE_CODE: &str = "XRP";

/// Codec for serializing and deserializing
/// vectors of Hash256.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency(Hash160);

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

    if candidate_iso == NATIVE_CODE {
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
fn _iso_to_bytes(value: &str) -> Result<[u8; CURRENCY_CODE_LENGTH], ISOCodeException> {
    if !_is_iso_code(value) {
        Err(ISOCodeException::InvalidISOCode)
    } else if value == NATIVE_CODE {
        Ok([0; CURRENCY_CODE_LENGTH])
    } else {
        let iso_bytes = value.as_bytes();
        let pad_left: [u8; 12] = [0; 12];
        let pad_right: [u8; 5] = [0; 5];
        let mut result: Vec<u8> = vec![];

        result.extend_from_slice(&pad_left);
        result.extend_from_slice(&iso_bytes);
        result.extend_from_slice(&pad_right);

        Ok(result
            .try_into()
            .or(Err(ISOCodeException::InvalidISOLength))?)
    }
}

impl Serializable for Currency {
    /// Construct a Currency.
    fn new(buffer: Option<&[u8]>) -> Result<Self, XRPLBinaryCodecException> {
        let hash160 = Hash160::new(buffer.or(Some(&[0; CURRENCY_CODE_LENGTH])))?;
        Ok(Currency(hash160))
    }

    /// Construct a Currency from a BinaryParser.
    fn from_parser(
        _: &mut BinaryParser,
        _: Option<usize>,
    ) -> Result<Currency, XRPLBinaryCodecException> {
        todo!()
    }
}

impl Buffered for Currency {
    fn get_buffer(&self) -> &[u8] {
        &self.0.get_buffer()
    }
}

impl TryFrom<&str> for Currency {
    type Error = ISOCodeException;

    /// Construct a Currency object from a string
    /// representation of a currency.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if _is_iso_code(value) {
            let hash160 = Hash160::new(Some(&_iso_to_bytes(value)?.to_vec()))?;
            Ok(Currency(hash160))
        } else if _is_hex(value) {
            Ok(Currency(Hash160::new(Some(&hex::decode(value)?))?))
        } else {
            Err(ISOCodeException::UnsupportedCurrencyRepresentation)
        }
    }
}

// TODO ToString on Bufferred does not work.
impl ToString for Currency {
    fn to_string(&self) -> String {
        let buffer = self.0.get_buffer();

        if hex::encode(buffer) == NATIVE_HEX_CODE {
            NATIVE_CODE.to_string()
        } else {
            let iso = _iso_code_from_hex(buffer);

            if let Ok(code) = iso {
                code.or_else(|| Some(hex::encode(buffer))).unwrap()
            } else {
                hex::encode(buffer)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const ILLEGAL_NATIVE_HEX_CODE: &str = "0000000000000000000000005852500000000000";
    const USD_HEX_CODE: &str = "0000000000000000000000005553440000000000";
    const NONSTANDARD_HEX_CODE: &str = "015841551A748AD2C1F76FF6ECB0CCCD00000000";
    const USD_ISO: &str = "USD";

    #[test]
    fn test_is_iso_code() {
        let valid_code = "ABC";
        let valid_code_numeric = "123";
        let invalid_code_long = "LONG";
        let invalid_code_short = "NO";

        assert!(_is_iso_code(valid_code));
        assert!(_is_iso_code(valid_code_numeric));
        assert!(!_is_iso_code(invalid_code_long));
        assert!(!_is_iso_code(invalid_code_short));
    }

    #[test]
    fn test_is_hex() {
        // Valid = 40 char length and only valid hex chars
        let valid_hex: &str = "0000000000000000000000005553440000000000";
        let invalid_hex_chars: &str = "USD0000000000000000000005553440000000000";
        let invalid_hex_long: &str = "0000000000000000000000005553440000000000123455";
        let invalid_hex_short: &str = "1234";

        assert!(_is_hex(valid_hex));
        assert!(!_is_hex(invalid_hex_long));
        assert!(!_is_hex(invalid_hex_short));
        assert!(!_is_hex(invalid_hex_chars));
    }

    #[test]
    fn test_iso_to_bytes() {
        // Valid non-XRP
        let usd_iso_bytes = _iso_to_bytes(USD_ISO).unwrap();
        // Valid XRP
        let xrp_iso_bytes = _iso_to_bytes(NATIVE_CODE).unwrap();
        // Error case
        let invalid_iso = "INVALID";

        assert_eq!(USD_HEX_CODE, hex::encode(usd_iso_bytes));
        assert_eq!(NATIVE_HEX_CODE, hex::encode(xrp_iso_bytes));
        assert!(_iso_to_bytes(invalid_iso).is_err());
    }

    // TODO
    // #[test]
    // fn test_currency_try_from() {
    //     let from_hex_xrp = Currency::try_from(NATIVE_HEX_CODE).unwrap();
    //     let from_hex_ic = Currency::try_from(USD_HEX_CODE).unwrap();
    //     let from_iso_xrp = Currency::try_from(NATIVE_CODE).unwrap();
    //     let from_iso_ic = Currency::try_from(USD_ISO).unwrap();
    //     let from_ns = Currency::try_from(NONSTANDARD_HEX_CODE).unwrap();

    //     assert_eq!(NATIVE_CODE, from_hex_xrp.to_string());
    //     assert_eq!(USD_ISO, from_hex_ic.to_string());
    //     assert_eq!(NATIVE_HEX_CODE, from_iso_xrp.to_string());
    //     assert_eq!(USD_HEX_CODE, from_iso_ic.to_string());
    //     assert_eq!(NONSTANDARD_HEX_CODE, from_ns.to_string());
    // }
}
