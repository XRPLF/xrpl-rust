//! Base class for XRPL Hash types.
//!
//! See Hash Fields:
//! `<https://xrpl.org/serialization.html#hash-fields>`

use crate::core::types::exceptions::XRPLHashException;
use crate::core::types::utils::*;
use crate::core::types::*;
use crate::core::BinaryParser;
use crate::core::Parser;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::Deserialize;

/// Codec for serializing and deserializing a hash field
/// with a width of 128 bits (16 bytes).
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Hash128(Vec<u8>);

/// Codec for serializing and deserializing a hash field
/// with a width of 160 bits (20 bytes).
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Hash160(Vec<u8>);

/// Codec for serializing and deserializing a hash field
/// with a width of 256 bits (32 bytes).
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Hash256(Vec<u8>);

/// XRPL Hash type.
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::types::hash::Hash;
///
/// #[derive(Debug)]
/// pub struct Hash256(Vec<u8>);
///
/// const _HASH256_LENGTH: usize = 16;
///
/// impl Hash for Hash256 {
///     fn get_length() -> usize {
///         _HASH256_LENGTH
///     }
/// }
/// ```
pub trait Hash {
    /// Get the length of the hash.
    fn get_length() -> usize
    where
        Self: Sized;
}

impl dyn Hash {
    /// Make a new hash of type T. Useful for extending
    /// new Hash lengths like Hash160.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::types::exceptions::XRPLHashException;
    /// use xrpl::core::types::hash::Hash;
    /// use xrpl::core::types::hash::Hash160;
    ///
    /// fn handle_success_case(hash: Vec<u8>) {
    ///     assert!(true)
    /// }
    ///
    /// fn handle_hash_error(error: XRPLHashException) {
    ///     // Error Conditions
    ///     match error {
    ///         XRPLHashException::InvalidHashLength {
    ///             expected,
    ///             found,
    ///         } => assert!(true),
    ///         _ => assert!(true),
    ///     }
    /// }
    ///
    /// let buffer: &str = "1000000000200000000030000000004000000000";
    /// let data: Vec<u8> = hex::decode(buffer).expect("");
    /// let result: Result<Vec<u8>, XRPLHashException> =
    ///     <dyn Hash>::make::<Hash160>(Some(&data));
    ///
    /// match result {
    ///     Ok(hash) => handle_success_case(hash),
    ///     Err(e) => handle_hash_error(e),
    /// };
    /// ```
    pub fn make<T: Hash>(bytes: Option<&[u8]>) -> Result<Vec<u8>, XRPLHashException> {
        let byte_value: &[u8] = bytes.unwrap_or(&[]);
        let hash_length: usize = T::get_length();

        if byte_value.len() != hash_length {
            Err(XRPLHashException::InvalidHashLength {
                expected: hash_length,
                found: byte_value.len(),
            })
        } else {
            Ok(byte_value.to_vec())
        }
    }

    /// Parse a hash type from a binary parser.
    ///
    /// # Examples
    ///
    /// ## Basic usage
    ///
    /// ```
    /// use xrpl::core::types::exceptions::XRPLHashException;
    /// use xrpl::core::binarycodec::exceptions::XRPLBinaryCodecException;
    /// use xrpl::core::binarycodec::BinaryParser;
    /// use xrpl::core::types::TryFromParser;
    /// use xrpl::core::types::hash::Hash;
    /// use xrpl::core::types::hash::Hash128;
    ///
    /// fn handle_success_case(hash: Hash128) {
    ///     // Success Conditions
    ///     assert!(true);
    /// }
    ///
    /// fn handle_parser_error(error: XRPLHashException) {
    ///     // Error Conditions
    ///     match error {
    ///         XRPLHashException::XRPLBinaryCodecError(_e) => assert!(false),
    ///         _ => assert!(false),
    ///     }
    /// }
    ///
    /// fn handle_hash128_from_parser(data: &[u8]) {
    ///     let mut parser: BinaryParser = BinaryParser::from(data);
    ///     let result: Result<Hash128, XRPLHashException> =
    ///         Hash128::from_parser(&mut parser, None);
    ///
    ///     match result {
    ///         Ok(hash) => handle_success_case(hash),
    ///         Err(e) => handle_parser_error(e)
    ///     }
    /// }
    ///
    /// let buffer: &str = "10000000002000000000300000000012";
    /// let data: Vec<u8> = hex::decode(buffer).expect("");
    ///
    /// handle_hash128_from_parser(&data);
    /// ```
    pub fn parse<T: Hash>(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Vec<u8>, XRPLHashException> {
        let read_length = length.or_else(|| Some(T::get_length())).unwrap();
        Ok(parser.read(read_length)?)
    }
}

impl Hash for Hash128 {
    fn get_length() -> usize {
        HASH128_LENGTH
    }
}

impl Hash for Hash160 {
    fn get_length() -> usize {
        HASH160_LENGTH
    }
}

impl Hash for Hash256 {
    fn get_length() -> usize {
        HASH256_LENGTH
    }
}

impl XRPLType for Hash128 {
    type Error = XRPLHashException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Hash128(<dyn Hash>::make::<Hash128>(buffer)?))
    }
}

impl XRPLType for Hash160 {
    type Error = XRPLHashException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Hash160(<dyn Hash>::make::<Hash160>(buffer)?))
    }
}

impl XRPLType for Hash256 {
    type Error = XRPLHashException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Hash256(<dyn Hash>::make::<Hash256>(buffer)?))
    }
}

impl TryFromParser for Hash128 {
    type Error = XRPLHashException;

    /// Build Hash128 from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Hash128, Self::Error> {
        Ok(Hash128(<dyn Hash>::parse::<Hash128>(parser, length)?))
    }
}

impl TryFromParser for Hash160 {
    type Error = XRPLHashException;

    /// Build Hash160 from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Hash160, Self::Error> {
        Ok(Hash160(<dyn Hash>::parse::<Hash160>(parser, length)?))
    }
}

impl TryFromParser for Hash256 {
    type Error = XRPLHashException;

    /// Build Hash256 from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Hash256, Self::Error> {
        Ok(Hash256(<dyn Hash>::parse::<Hash256>(parser, length)?))
    }
}

impl TryFrom<&str> for Hash128 {
    type Error = XRPLHashException;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hash128::new(Some(&hex::decode(value)?))
    }
}

impl TryFrom<&str> for Hash160 {
    type Error = XRPLHashException;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hash160::new(Some(&hex::decode(value)?))
    }
}

impl TryFrom<&str> for Hash256 {
    type Error = XRPLHashException;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hash256::new(Some(&hex::decode(value)?))
    }
}

impl ToString for Hash128 {
    /// Get the hex representation of the Hash128 bytes.
    fn to_string(&self) -> String {
        hex::encode_upper(self.as_ref())
    }
}

impl ToString for Hash160 {
    /// Get the hex representation of the Hash160 bytes.
    fn to_string(&self) -> String {
        hex::encode_upper(self.as_ref())
    }
}

impl ToString for Hash256 {
    /// Get the hex representation of the Hash256 bytes.
    fn to_string(&self) -> String {
        hex::encode_upper(self.as_ref())
    }
}

impl AsRef<[u8]> for Hash160 {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash128 {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash256 {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HASH128_HEX_TEST: &str = "10000000002000000000300000000012";
    const HASH160_HEX_TEST: &str = "1000000000200000000030000000004000000000";
    const HASH256_HEX_TEST: &str =
        "1000000000200000000030000000004000000000500000000060000000001234";

    #[test]
    fn test_hash_new() {
        let hex128 = hex::decode(HASH128_HEX_TEST).unwrap();
        let hex160 = hex::decode(HASH160_HEX_TEST).unwrap();
        let hex256 = hex::decode(HASH256_HEX_TEST).unwrap();

        assert_eq!(HASH128_HEX_TEST, Hash128(hex128).to_string());
        assert_eq!(HASH160_HEX_TEST, Hash160(hex160).to_string());
        assert_eq!(HASH256_HEX_TEST, Hash256(hex256).to_string());
    }

    #[test]
    fn test_hash_try_from_parser() {
        let hex = hex::decode(HASH128_HEX_TEST).expect("");
        let mut parser = BinaryParser::from(hex);
        let result = Hash128::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(HASH128_HEX_TEST, result.unwrap().to_string());

        let hex = hex::decode(HASH160_HEX_TEST).expect("");
        let mut parser = BinaryParser::from(hex);
        let result = Hash160::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(HASH160_HEX_TEST, result.unwrap().to_string());

        let hex = hex::decode(HASH256_HEX_TEST).expect("");
        let mut parser = BinaryParser::from(hex);
        let result = Hash256::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(HASH256_HEX_TEST, result.unwrap().to_string());
    }

    #[test]
    fn test_hash_try_from() {
        let result = Hash128::try_from(HASH128_HEX_TEST);

        assert!(result.is_ok());
        assert_eq!(HASH128_HEX_TEST, result.unwrap().to_string());

        let result = Hash160::try_from(HASH160_HEX_TEST);

        assert!(result.is_ok());
        assert_eq!(HASH160_HEX_TEST, result.unwrap().to_string());

        let result = Hash256::try_from(HASH256_HEX_TEST);

        assert!(result.is_ok());
        assert_eq!(HASH256_HEX_TEST, result.unwrap().to_string());
    }

    #[test]
    fn accept_hash_invalid_length_errors() {
        let hash128 = Hash128::try_from("1000000000200000000030000000001234");
        let hash160 = Hash160::try_from("100000000020000000003000000000400000000012");
        let hash256 =
            Hash256::try_from("100000000020000000003000000000400000000050000000006000000000123456");

        assert!(hash128.is_err());
        assert!(hash160.is_err());
        assert!(hash256.is_err());
    }
}
