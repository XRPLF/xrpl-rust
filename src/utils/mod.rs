//! Convenience utilities for the XRP Ledger

pub mod exceptions;
#[cfg(feature = "models")]
pub mod get_nftoken_id;
pub mod get_xchain_claim_id;
#[cfg(feature = "models")]
pub mod get_xchain_claim_id;
#[cfg(feature = "models")]
pub mod parse_nftoken_id;
pub mod str_conversion;
pub mod time_conversion;
#[cfg(feature = "models")]
pub(crate) mod transactions;
#[cfg(feature = "models")]
pub mod txn_parser;
pub mod xrpl_conversion;

pub use self::time_conversion::*;
pub use self::xrpl_conversion::*;

use crate::constants::*;
use alloc::vec::Vec;
use regex::Regex;

/// Determine if the address string is a hex address.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::is_hex_address;
///
/// let value: &str = "5E7B112523F68D2F5E879DB4EAC51C6698A69304";
///
/// assert!(is_hex_address(value));
/// ```
pub fn is_hex_address(value: &str) -> bool {
    let regex = Regex::new(HEX_CURRENCY_REGEX).expect("is_hex_address");
    regex.is_match(value)
}

/// Tests if value is a valid 3-char iso code.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::is_iso_code;
///
/// let value: &str = "USD";
///
/// assert!(is_iso_code(value));
/// ```
pub fn is_iso_code(value: &str) -> bool {
    let regex = Regex::new(ISO_CURRENCY_REGEX).expect("is_iso_code");
    regex.is_match(value)
}

/// Tests if value is a valid 40-char hex currency string.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::is_iso_hex;
///
/// let value: &str = "0000000000000000000000005553440000000000";
///
/// assert!(is_iso_hex(value));
/// ```
pub fn is_iso_hex(value: &str) -> bool {
    let regex = Regex::new(HEX_CURRENCY_REGEX).expect("_is_hex");
    regex.is_match(value)
}

/// Converter to byte array with endianness.
pub trait ToBytes {
    /// Return the byte array of self.
    fn to_bytes(&self) -> Vec<u8>;
}

#[cfg(test)]
mod test {
    use super::*;

    const HEX_ENCODING: &str = "5E7B112523F68D2F5E879DB4EAC51C6698A69304";

    #[test]
    fn test_is_hex_address() {
        assert!(is_hex_address(HEX_ENCODING));
    }

    #[test]
    fn test_is_iso_code() {
        let valid_code = "ABC";
        let valid_code_numeric = "123";
        let invalid_code_long = "LONG";
        let invalid_code_short = "NO";

        assert!(is_iso_code(valid_code));
        assert!(is_iso_code(valid_code_numeric));
        assert!(!is_iso_code(invalid_code_long));
        assert!(!is_iso_code(invalid_code_short));
    }

    #[test]
    fn test_is_hex() {
        // Valid = 40 char length and only valid hex chars
        let valid_hex: &str = "0000000000000000000000005553440000000000";
        let invalid_hex_chars: &str = "USD0000000000000000000005553440000000000";
        let invalid_hex_long: &str = "0000000000000000000000005553440000000000123455";
        let invalid_hex_short: &str = "1234";

        assert!(is_iso_hex(valid_hex));
        assert!(!is_iso_hex(invalid_hex_long));
        assert!(!is_iso_hex(invalid_hex_short));
        assert!(!is_iso_hex(invalid_hex_chars));
    }
}
