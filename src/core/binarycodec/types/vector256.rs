//! Codec for serializing and deserializing
//! vectors of Hash256.

use crate::core::binarycodec::types::exceptions::XRPLVectorException;
use crate::core::binarycodec::types::hash::Hash256;
use crate::core::binarycodec::types::*;
use crate::core::BinaryParser;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::ser::Error;
use serde::ser::SerializeSeq;
use serde::Serializer;
use serde::{Deserialize, Serialize};

const _HASH_LENGTH_BYTES: usize = 32;

/// Codec for serializing and deserializing
/// vectors of Hash256.
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "Vec<&str>")]
pub struct Vector256(Vec<u8>);

impl XRPLType for Vector256 {
    type Error = XRPLVectorException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(Vector256(data.to_vec()))
        } else {
            Ok(Vector256(vec![]))
        }
    }
}

impl TryFromParser for Vector256 {
    type Error = XRPLVectorException;

    /// Build Vector256 from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Vector256, Self::Error> {
        let mut bytes = vec![];

        let num_bytes: usize = if let Some(value) = length {
            value
        } else {
            parser.len()
        };

        let num_hashes: usize = num_bytes / _HASH_LENGTH_BYTES;

        for _ in 0..num_hashes {
            bytes.extend_from_slice(Hash256::from_parser(parser, None)?.as_ref());
        }

        Ok(Vector256(bytes))
    }
}

impl Serialize for Vector256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.len() % _HASH_LENGTH_BYTES != 0 {
            Err(S::Error::custom(XRPLVectorException::InvalidVector256Bytes))
        } else {
            let mut sequence = serializer.serialize_seq(None)?;

            for i in (0..self.0.len()).step_by(_HASH_LENGTH_BYTES) {
                let encoded = hex::encode_upper(&self.0[i..i + _HASH_LENGTH_BYTES]);
                sequence.serialize_element(&encoded)?;
            }

            sequence.end()
        }
    }
}

impl TryFrom<Vec<&str>> for Vector256 {
    type Error = XRPLVectorException;

    /// Construct a Vector256 from a list of strings.
    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        let mut bytes = vec![];

        for string in value {
            bytes.extend_from_slice(Hash256::try_from(string)?.as_ref())
        }

        Ok(Vector256(bytes))
    }
}

impl Display for Vector256 {
    /// Get the hex representation of the Vector256 bytes.
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&hex::encode_upper(self.as_ref()), f)
    }
}

impl AsRef<[u8]> for Vector256 {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::format;

    const SERIALIZED: &str = "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373";
    const HASH1: &str = "42426C4D4F1009EE67080A9B7965B44656D7714D104A72F9B4369F97ABF044EE";
    const HASH2: &str = "4C97EBA926031A7CF7D7B36FDE3ED66DDA5421192D63DE53FFB46E43B9DC8373";

    #[test]
    fn test_vector256_new() {
        let bytes = hex::decode(HASH1).unwrap();
        assert_eq!(HASH1, Vector256(bytes).to_string());
    }

    #[test]
    fn test_vector256_try_from_parser() {
        let hex = hex::decode(SERIALIZED).expect("");
        let mut parser = BinaryParser::from(hex);
        let result = Vector256::from_parser(&mut parser, None);

        assert!(result.is_ok());
        assert_eq!(SERIALIZED, result.unwrap().to_string());
    }

    #[test]
    fn test_vector256_try_from() {
        let result = Vector256::try_from(vec![HASH1, HASH2]);

        assert!(result.is_ok());
        assert_eq!(SERIALIZED, result.unwrap().to_string());
    }

    #[test]
    fn accept_vector256_serde_encode_decode() {
        let vector = Vector256::try_from(vec![HASH1, HASH2]).unwrap();
        let serialize = serde_json::to_string(&vector).unwrap();
        let deserialize: Vector256 = serde_json::from_str(&serialize).unwrap();

        assert_eq!(format!("[\"{HASH1}\",\"{HASH2}\"]"), serialize);
        assert_eq!(SERIALIZED, deserialize.to_string());
    }
}
