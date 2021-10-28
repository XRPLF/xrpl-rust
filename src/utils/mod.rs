//! Convenience utilities for the XRP Ledger

pub mod exceptions;
pub mod time_conversion;
pub mod xrpl_conversion;

pub use self::time_conversion::*;
pub use self::xrpl_conversion::*;

use crate::constants::HEX_CURRENCY_REGEX;
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
}
