//! Codec for serializing and deserializing
//! vectors of Hash256.

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::types::hash256::Hash256;
use crate::core::binarycodec::types::serialized_type::Buffered;
use crate::core::binarycodec::types::serialized_type::Serializable;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::{Deserialize, Serialize};

const _HASH_LENGTH_BYTES: usize = 32;

/// Codec for serializing and deserializing
/// vectors of Hash256.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vector256(Vec<u8>);

impl Serializable for Vector256 {
    /// Construct a Vector256.
    fn new(buffer: Option<&[u8]>) -> Result<Self, XRPLBinaryCodecException> {
        Ok(Vector256(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }

    /// Construct a Vector256 from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Vector256, XRPLBinaryCodecException> {
        let mut bytes = vec![];
        let num_bytes: usize;
        let num_hashes: usize;

        if let Some(value) = length {
            num_bytes = value;
        } else {
            num_bytes = parser.len();
        }

        num_hashes = num_bytes / _HASH_LENGTH_BYTES;

        for _ in 0..num_hashes {
            bytes.extend_from_slice(Hash256::from_parser(parser, None)?.get_buffer());
        }

        Ok(Vector256(bytes))
    }
}

impl Buffered for Vector256 {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[&str]> for Vector256 {
    type Error = XRPLBinaryCodecException;

    /// Construct a Vector256 from a list of strings.
    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        let mut bytes = vec![];

        for string in value {
            bytes.extend_from_slice(Hash256::try_from(string.as_ref())?.get_buffer())
        }

        Ok(Vector256(bytes))
    }
}

// TODO ToString on Bufferred does not work.
impl ToString for Vector256 {
    fn to_string(&self) -> String {
        hex::encode(self.get_buffer()).to_uppercase()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SERIALIZED: &str = "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373";
    const HASH1: &str = "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE";
    const HASH2: &str = "4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373";
    const HASH_LIST: &[&str] = &[HASH1, HASH2];

    #[test]
    fn test_new() {
        let bytes = hex::decode(HASH1).unwrap();
        assert_eq!(HASH1, Vector256(bytes).to_string());
    }

    #[test]
    fn test_from_parser() {
        let mut parser = BinaryParser::from(hex::decode(SERIALIZED).unwrap());
        let result = Vector256::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(SERIALIZED, result.unwrap().to_string());
    }

    #[test]
    fn test_try_from() {
        let result = Vector256::try_from(HASH_LIST);

        assert!(result.is_ok());
        assert_eq!(SERIALIZED, result.unwrap().to_string());
    }
}
