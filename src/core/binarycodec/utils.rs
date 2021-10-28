//! Utilities for binarycodec crate.

use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::definitions::load_definition_map;
use crate::core::definitions::DefinitionHandler;
use crate::core::definitions::FieldHeader;
use crate::core::definitions::CODE_MAX_VALUE;
use crate::core::definitions::CODE_MIN_VALUE;
use alloc::vec;
use alloc::vec::Vec;

/// Max length that can be represented in a single byte
/// per XRPL serialization encoding.
pub const MAX_SINGLE_BYTE_LENGTH: usize = 192;
/// Max length that can be represented in 2 bytes per
/// XRPL serialization encoding.
pub const MAX_DOUBLE_BYTE_LENGTH: usize = 12481;
/// Max value that can be used in the second byte of a
/// length field.
pub const MAX_SECOND_BYTE_VALUE: usize = 240;
/// Max value that can be represented in using two
/// 8-bit bytes (2^16)
pub const MAX_DOUBLE_BYTE_VALUE: usize = 65536;
/// Maximum length that can be encoded in a length
/// prefix per XRPL serialization encoding.
pub const MAX_LENGTH_VALUE: usize = 918744;
/// Max value that can be represented using one 8-bit
/// byte (2^8)
pub const MAX_BYTE_VALUE: usize = 256;

/// See: `<https://xrpl.org/serialization.html#field-ids>`
fn _encode_field_id(field_header: &FieldHeader) -> Result<Vec<u8>, XRPLBinaryCodecException> {
    let type_code = field_header.type_code;
    let field_code = field_header.field_code;
    let range = CODE_MIN_VALUE..CODE_MAX_VALUE;

    if !range.contains(&field_code) {
        Err(XRPLBinaryCodecException::UnexpectedFieldCodeRange {
            min: CODE_MIN_VALUE as usize,
            max: CODE_MAX_VALUE as usize,
        })
    } else if !range.contains(&type_code) {
        Err(XRPLBinaryCodecException::UnexpectedTypeCodeRange {
            min: CODE_MIN_VALUE as usize,
            max: CODE_MAX_VALUE as usize,
        })
    } else if type_code < 16 && field_code < 16 {
        // high 4 bits is the type_code
        // low 4 bits is the field code
        let combined_code = (type_code << 4) | field_code;
        Ok([combined_code as u8].to_vec())
    } else if type_code >= 16 && field_code < 16 {
        // first 4 bits are zeroes
        // next 4 bits is field code
        // next byte is type code
        let mut result = vec![];
        let byte1 = [field_code as u8];
        let byte2 = [type_code as u8];

        result.extend_from_slice(&byte1);
        result.extend_from_slice(&byte2);

        Ok(result)
    } else if type_code < 16 && field_code >= 16 {
        // first 4 bits is type code
        // next 4 bits are zeroes
        // next byte is field code
        let mut result = vec![];
        let byte1 = [(type_code << 4) as u8];
        let byte2 = [field_code as u8];

        result.extend_from_slice(&byte1);
        result.extend_from_slice(&byte2);

        Ok(result)
    } else {
        // both are >= 16
        // first byte is all zeroes
        // second byte is type code
        // third byte is field code
        let mut result = vec![];
        let byte2 = [type_code as u8];
        let byte3 = [field_code as u8];

        result.extend_from_slice(&[0]);
        result.extend_from_slice(&byte2);
        result.extend_from_slice(&byte3);

        Ok(result)
    }
}

/// See: `<https://xrpl.org/serialization.html#field-ids>`
fn _decode_field_id(field_id: &str) -> Result<FieldHeader, XRPLBinaryCodecException> {
    let bytes = hex::decode(field_id)?;

    match bytes.len() {
        1 => {
            let type_code = (bytes[0] >> 4) as i16;
            let field_code = (bytes[0] & 0x0F) as i16;

            Ok(FieldHeader {
                type_code,
                field_code,
            })
        }
        2 => {
            let first_byte = bytes[0];
            let second_byte = bytes[1];
            let first_byte_high_bits = first_byte >> 4;
            let first_byte_low_bits = first_byte & 0x0F;

            if first_byte_high_bits == 0 {
                // Next 4 bits are field code, second byte
                // is type code.
                let type_code = second_byte as i16;
                let field_code = first_byte_low_bits as i16;

                Ok(FieldHeader {
                    type_code,
                    field_code,
                })
            } else {
                // Otherwise, next 4 bits are type code,
                // second byte is field code.
                let type_code = first_byte_high_bits as i16;
                let field_code = second_byte as i16;

                Ok(FieldHeader {
                    type_code,
                    field_code,
                })
            }
        }
        3 => {
            let type_code = bytes[1] as i16;
            let field_code = bytes[2] as i16;

            Ok(FieldHeader {
                type_code,
                field_code,
            })
        }
        _ => Err(XRPLBinaryCodecException::UnexpectedFieldIdByteRange { min: 1, max: 3 }),
    }
}

/// Returns the unique field ID for a given field name.
/// This field ID consists of the type code and field
/// code, in 1 to 3 bytes depending on whether those
/// values are "common" (<16) or "uncommon" (>=16)
///
/// See Field Ids:
/// `<https://xrpl.org/serialization.html#field-ids>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::binarycodec::utils::encode_field_name;
/// use xrpl::core::binarycodec::exceptions::XRPLBinaryCodecException;
/// extern crate alloc;
/// use alloc::vec;
///
/// let field_name: &str = "LedgerSequence";
/// let bytes: Vec<u8> = vec![38];
///
/// let encoding: Option<Vec<u8>> = match encode_field_name(field_name) {
///     Ok(bytes) => Some(bytes),
///     Err(e) => match e {
///         XRPLBinaryCodecException::UnknownFieldName => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(bytes), encoding);
/// ```
pub fn encode_field_name(field_name: &str) -> Result<Vec<u8>, XRPLBinaryCodecException> {
    let definitions = load_definition_map();
    let field_header = definitions.get_field_header_from_name(field_name);

    if let Some(header) = field_header {
        _encode_field_id(&header)
    } else {
        Err(XRPLBinaryCodecException::UnknownFieldName)
    }
}

/// Returns the field name represented by the given field ID.
///
/// See Field Ids:
/// `<https://xrpl.org/serialization.html#field-ids>`
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::core::binarycodec::utils::decode_field_name;
/// use xrpl::core::binarycodec::exceptions::XRPLBinaryCodecException;
///
/// let field_id: &str = "26";
/// let field_name: &str = "LedgerSequence";
///
/// let decoding: Option<&str> = match decode_field_name(field_id) {
///     Ok(field_name) => Some(field_name),
///     Err(e) => match e {
///         XRPLBinaryCodecException::UnexpectedFieldIdByteRange {
///             min: _,
///             max: _
///         } => None,
///         _ => None,
///     }
/// };
///
/// assert_eq!(Some(field_name), decoding);
/// ```
pub fn decode_field_name(field_id: &str) -> Result<&str, XRPLBinaryCodecException> {
    let definitions = load_definition_map();
    let field_header = _decode_field_id(field_id)?;
    let field_name = definitions.get_field_name_from_header(&field_header);

    if let Some(name) = field_name {
        Ok(name)
    } else {
        Err(XRPLBinaryCodecException::UnknownFieldName)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::binarycodec::test_cases::load_field_tests;

    #[test]
    fn test_encode_field_name() {
        for test in load_field_tests() {
            let result = hex::encode_upper(encode_field_name(&test.name).expect(""));
            assert_eq!(test.expected_hex, result)
        }
    }

    #[test]
    fn test_decode_field_name() {
        for test in load_field_tests() {
            assert_eq!(
                decode_field_name(&test.expected_hex),
                Ok(test.name.as_ref())
            )
        }
    }
}
