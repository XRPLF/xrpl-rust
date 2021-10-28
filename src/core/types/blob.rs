//! Codec for serializing and deserializing blob fields.
//!
//! See Blob Fields:
//! `<https://xrpl.org/serialization.html#blob-fields>`

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::types::*;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::Serializer;
use serde::{Deserialize, Serialize};

/// Codec for serializing and deserializing blob fields.
///
/// See Blob Fields:
/// `<https://xrpl.org/serialization.html#blob-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Blob(Vec<u8>);

impl XRPLType for Blob {
    type Error = XRPLBinaryCodecException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(Blob(data.to_vec()))
        } else {
            Ok(Blob(vec![]))
        }
    }
}

impl Serialize for Blob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode_upper(self.as_ref()))
    }
}

impl TryFrom<&str> for Blob {
    type Error = XRPLBinaryCodecException;

    /// Construct a Blob from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(Some(&hex::decode(value)?))
    }
}

impl ToString for Blob {
    /// Get the hex representation of the Blob bytes.
    fn to_string(&self) -> String {
        hex::encode_upper(self.as_ref())
    }
}

impl AsRef<[u8]> for Blob {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_HEX: &str = "00AA";

    #[test]
    fn test_blob_new() {
        let bytes = hex::decode(TEST_HEX).unwrap();
        let blob = Blob::new(Some(&bytes));

        assert!(blob.is_ok());
        assert_eq!(bytes, blob.unwrap().as_ref());
    }

    #[test]
    fn test_blob_try_from() {
        let bytes = hex::decode(TEST_HEX).unwrap();
        let blob = Blob::try_from(TEST_HEX);

        assert!(blob.is_ok());
        assert_eq!(bytes, blob.unwrap().as_ref());
    }
}
