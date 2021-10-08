//! Context manager and helpers for the serialization
//! of a JSON object into bytes.

use crate::core::binarycodec::binary_wrappers::utils::MAX_DOUBLE_BYTE_LENGTH;
use crate::core::binarycodec::binary_wrappers::utils::MAX_LENGTH_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SECOND_BYTE_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SINGLE_BYTE_LENGTH;
use crate::core::binarycodec::definitions::field_instance::FieldInstance;
use crate::core::binarycodec::exceptions::VariableLengthException;
use crate::to_bytes;
use crate::utils::byte_conversions::ToBytes;

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
/// See Length Prefixing: https://xrpl.org/serialization.html#length-prefixing
fn _encode_variable_length_prefix(length: &usize) -> Result<Vec<u8>, VariableLengthException> {
    if length <= &MAX_SINGLE_BYTE_LENGTH {
        Ok(to_bytes!(length).to_vec())
    } else if length < &MAX_DOUBLE_BYTE_LENGTH {
        let mut bytes = vec![];
        let b_length = *length - MAX_SINGLE_BYTE_LENGTH + 1;
        let val_a = (b_length >> 8) + (MAX_SINGLE_BYTE_LENGTH + 1);
        let val_b = b_length & 0xFF;

        bytes.extend_from_slice(&to_bytes!(val_a));
        bytes.extend_from_slice(&to_bytes!(val_b));

        Ok(bytes)
    } else if length <= &MAX_LENGTH_VALUE {
        let mut bytes = vec![];
        let b_length = *length - MAX_DOUBLE_BYTE_LENGTH;
        let val_a = (MAX_SECOND_BYTE_VALUE + 1) + (b_length >> 16);
        let val_b = (b_length >> 8) & 0xFF;
        let val_c = b_length & 0xFF;

        bytes.extend_from_slice(&to_bytes!(val_a));
        bytes.extend_from_slice(&to_bytes!(val_b));
        bytes.extend_from_slice(&to_bytes!(val_c));

        Ok(bytes)
    } else {
        Err(VariableLengthException::new(&format!(
            "VariableLength field must be <= {} bytes long",
            MAX_LENGTH_VALUE
        )))
    }
}

pub trait Serializer {
    /// Write a variable length encoded value to
    /// the BinarySerializer.
    fn write_length_encoded(&mut self, value: &[u8]) -> &Self;

    /// Write field and value to the buffer.
    fn write_field_and_value(&mut self, field: FieldInstance, value: &[u8]) -> &Self;
}

impl Serializer for BinarySerializer {
    fn write_length_encoded(&mut self, value: &[u8]) -> &Self {
        let length_prefix = _encode_variable_length_prefix(&value.len());

        self.extend_from_slice(&length_prefix.unwrap());
        self.extend_from_slice(&value);

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

    #[test]
    fn test_encode_variable_length_prefix() {
        let length_a = 192;
        let length_b = 12480;
        let length_c = 918744;
        println!("{:?}", _encode_variable_length_prefix(&length_a));
        println!("{:?}", _encode_variable_length_prefix(&length_b));
        println!("{:?}", _encode_variable_length_prefix(&length_c));
        //assert_eq!(vec![], _encode_variable_length_prefix(&length_a).unwrap());
    }
}
