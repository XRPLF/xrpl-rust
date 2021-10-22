//! Codec for serializing and deserializing a hash
//! field with a width of 160 bits (20 bytes).
//!
//! See Hash Fields:
//! `<https://xrpl.org/serialization.html#hash-fields>`

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::types::hash::Hash;
use crate::core::binarycodec::types::xrpl_type::Buffered;
use crate::core::binarycodec::types::xrpl_type::FromParser;
use crate::core::binarycodec::types::xrpl_type::XRPLType;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::Deserialize;

/// Codec for serializing and deserializing a hash field
/// with a width of 160 bits (20 bytes).
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Hash160(Vec<u8>);

const _HASH160_LENGTH: usize = 20;

impl Hash for Hash160 {
    fn get_length() -> usize {
        _HASH160_LENGTH
    }
}

impl XRPLType for Hash160 {
    type Error = XRPLBinaryCodecException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Hash160(<dyn Hash>::make::<Hash160>(buffer)?))
    }
}

impl FromParser for Hash160 {
    type Error = XRPLBinaryCodecException;

    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Hash160, Self::Error> {
        Ok(Hash160(<dyn Hash>::parse::<Hash160>(parser, length)?))
    }
}

impl Buffered for Hash160 {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&str> for Hash160 {
    type Error = XRPLBinaryCodecException;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hash160::new(Some(&hex::decode(value)?))
    }
}

// TODO ToString on Bufferred does not work.
impl ToString for Hash160 {
    fn to_string(&self) -> String {
        hex::encode(self.get_buffer())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_HEX: &str = "1000000000200000000030000000004000000000";

    #[test]
    fn test_new() {
        let bytes = hex::decode(TEST_HEX).unwrap();
        assert_eq!(TEST_HEX, Hash160(bytes).to_string());
    }

    #[test]
    fn test_from_parser() {
        let mut parser = BinaryParser::from(hex::decode(TEST_HEX).unwrap());
        let result = Hash160::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(TEST_HEX, result.unwrap().to_string());
    }

    #[test]
    fn test_try_from() {
        let result = Hash160::try_from(TEST_HEX);

        assert!(result.is_ok());
        assert_eq!(TEST_HEX, result.unwrap().to_string());
    }

    #[test]
    fn accept_invalid_length_errors() {
        let result = Hash160::try_from("100000000020000000003000000000400000000012");
        assert!(result.is_err());
    }
}
