//! Context manager and helpers for the serialization
//! of a JSON object into bytes.

use crate::core::binarycodec::binary_wrappers::utils::MAX_DOUBLE_BYTE_LENGTH;
use crate::core::binarycodec::binary_wrappers::utils::MAX_LENGTH_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SECOND_BYTE_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SINGLE_BYTE_LENGTH;
use crate::core::binarycodec::definitions::field_instance::FieldInstance;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::utils::byte_conversions::ToBytes;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;

/// Serializes JSON to XRPL binary format.
pub type BinarySerializer = Vec<u8>;

/// Helper function for length-prefixed fields including
/// Blob types and some AccountID types. Calculates the
/// prefix of variable length bytes.
///
/// The length of the prefix is 1-3 bytes depending on the
/// length of the contents:
/// Content length <= 192 bytes: prefix is 1 byte
/// 192 bytes < Content length <= 12480 bytes: prefix is 2 bytes
/// 12480 bytes < Content length <= 918744 bytes: prefix is 3 bytes
///
/// See Length Prefixing: `<https://xrpl.org/serialization.html#length-prefixing>`
fn _encode_variable_length_prefix(length: &usize) -> Result<Vec<u8>, XRPLBinaryCodecException> {
    if length <= &MAX_SINGLE_BYTE_LENGTH {
        Ok([*length as u8].to_vec())
    } else if length < &MAX_DOUBLE_BYTE_LENGTH {
        let mut bytes = vec![];
        let b_length = *length - (MAX_SINGLE_BYTE_LENGTH + 1);
        let val_a: u8 = ((b_length >> 8) + (MAX_SINGLE_BYTE_LENGTH + 1))
            .try_into()
            .unwrap();
        let val_b: u8 = (b_length & 0xFF).try_into().unwrap();

        bytes.extend_from_slice(&[val_a]);
        bytes.extend_from_slice(&[val_b]);

        Ok(bytes)
    } else if length <= &MAX_LENGTH_VALUE {
        let mut bytes = vec![];
        let b_length = *length - MAX_DOUBLE_BYTE_LENGTH;
        let val_a: u8 = ((MAX_SECOND_BYTE_VALUE + 1) + (b_length >> 16))
            .try_into()
            .unwrap();
        let val_b: u8 = ((b_length >> 8) & 0xFF).try_into().unwrap();
        let val_c: u8 = (b_length & 0xFF).try_into().unwrap();

        bytes.extend_from_slice(&[val_a]);
        bytes.extend_from_slice(&[val_b]);
        bytes.extend_from_slice(&[val_c]);

        Ok(bytes)
    } else {
        Err(XRPLBinaryCodecException::InvalidVariableLengthTooLarge {
            max: MAX_LENGTH_VALUE,
        })
    }
}

pub trait Serialization {
    /// Write given bytes to this BinarySerializer.
    fn append(&mut self, bytes: &[u8]) -> &Self;

    /// Write a variable length encoded value to
    /// the BinarySerializer.
    fn write_length_encoded(&mut self, value: &[u8]) -> &Self;

    /// Write field and value to the buffer.
    fn write_field_and_value(&mut self, field: FieldInstance, value: &[u8]) -> &Self;
}

impl Serialization for BinarySerializer {
    fn append(&mut self, bytes: &[u8]) -> &Self {
        self.extend_from_slice(bytes);
        self
    }

    fn write_length_encoded(&mut self, value: &[u8]) -> &Self {
        let length_prefix = _encode_variable_length_prefix(&value.len());

        self.extend_from_slice(&length_prefix.unwrap());
        self.extend_from_slice(value);

        self
    }

    fn write_field_and_value(&mut self, field: FieldInstance, value: &[u8]) -> &Self {
        self.extend_from_slice(&field.header.to_bytes());

        if field.is_vl_encoded {
            self.write_length_encoded(value);
        } else {
            self.extend_from_slice(value);
        }

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
    use crate::core::binarycodec::binary_wrappers::binary_parser::Parser;
    use alloc::string::String;

    /// This is currently a sanity check for private
    /// [`_encode_variable_length_prefix`], which is called by
    /// BinarySerializer.write_length_encoded.
    #[test]
    fn test_encode_variable_length_prefix() {
        for case in [100_usize, 1000, 20_000] {
            let blob = (0..case).map(|_| "A2").collect::<String>();
            let mut binary_serializer: BinarySerializer = BinarySerializer::new();

            binary_serializer.write_length_encoded(&hex::decode(blob).unwrap());

            let mut binary_parser: BinaryParser = BinaryParser::from(binary_serializer.as_ref());
            let decoded_length = binary_parser.read_length_prefix();

            assert!(decoded_length.is_ok());
            assert_eq!(case, decoded_length.unwrap());
        }
    }
}
