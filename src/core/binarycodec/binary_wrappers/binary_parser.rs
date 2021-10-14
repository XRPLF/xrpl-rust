//! Context manager and helpers for the deserialization
//! of bytes into JSON.

use crate::core::binarycodec::binary_wrappers::utils::MAX_BYTE_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_DOUBLE_BYTE_LENGTH;
use crate::core::binarycodec::binary_wrappers::utils::MAX_DOUBLE_BYTE_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SECOND_BYTE_VALUE;
use crate::core::binarycodec::binary_wrappers::utils::MAX_SINGLE_BYTE_LENGTH;
use crate::core::binarycodec::definitions::definition_types::get_field_instance;
use crate::core::binarycodec::definitions::definition_types::get_field_name_from_header;
use crate::core::binarycodec::definitions::field_header::FieldHeader;
use crate::core::binarycodec::definitions::field_instance::FieldInstance;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::vec::Vec;
use core::convert::TryInto;

/// Deserializes from hex-encoded XRPL binary format to
/// serde JSON fields and values.
#[derive(Debug)]
pub struct BinaryParser(Vec<u8>);

pub trait Parser {
    /// Peek the first byte of the BinaryParser.
    fn peek(&self) -> Option<[u8; 1]>;

    /// Consume the first n bytes of the BinaryParser.
    fn skip(&mut self, n: usize) -> Result<&Self, XRPLBinaryCodecException>;

    /// Consume and return the first n bytes of the
    /// BinaryParser.
    fn read(&mut self, n: usize) -> Result<Vec<u8>, XRPLBinaryCodecException>;

    /// Read 1 byte from parser and return as
    /// unsigned int.
    fn read_uint8(&mut self) -> Result<u8, XRPLBinaryCodecException>;

    /// Read 2 bytes from parser and return as
    /// unsigned int.
    fn read_uint16(&mut self) -> Result<u16, XRPLBinaryCodecException>;

    /// Read 4 bytes from parser and return as
    /// unsigned int.
    fn read_uint32(&mut self) -> Result<u32, XRPLBinaryCodecException>;

    /// Returns whether the binary parser has finished
    /// parsing (e.g. there is nothing left in the buffer
    /// that needs to be processed).
    fn is_end(&self, custom_end: Option<usize>) -> bool;

    /// Reads a variable length encoding prefix and
    /// returns the encoded length.
    /// The formula for decoding a length prefix is
    /// described in Length Prefixing:
    /// `<https://xrpl.org/serialization.html#length-prefixing>`
    fn read_length_prefix(&mut self) -> Result<usize, XRPLBinaryCodecException>;

    /// Reads field ID from BinaryParser and returns as
    /// a FieldHeader object.
    fn read_field_header(&mut self) -> Result<FieldHeader, XRPLBinaryCodecException>;

    /// Read the field ordinal at the head of the
    /// BinaryParser and return a FieldInstance object
    /// representing information about the field
    /// containedin the following bytes.
    fn read_field(&mut self) -> Result<FieldInstance, XRPLBinaryCodecException>;

    // TODO Implement type methods
}

/// Peek the first byte of the BinaryParser.
impl Parser for BinaryParser {
    fn peek(&self) -> Option<[u8; 1]> {
        if !self.0.is_empty() {
            Some(self.0[0].to_be_bytes())
        } else {
            None
        }
    }

    fn skip(&mut self, n: usize) -> Result<&Self, XRPLBinaryCodecException> {
        if n > self.0.len() {
            Err(XRPLBinaryCodecException::new(&format!(
                "BinaryParser can't skip {} bytes, only contains {}.",
                n,
                self.0.len()
            )))
        } else {
            self.0 = self.0[n..].to_vec();
            Ok(self)
        }
    }

    fn read(&mut self, n: usize) -> Result<Vec<u8>, XRPLBinaryCodecException> {
        let first_n_bytes = self.0[..n].to_owned();

        self.skip(n)?;
        Ok(first_n_bytes)
    }

    fn read_uint8(&mut self) -> Result<u8, XRPLBinaryCodecException> {
        let result = self.read(1)?;
        Ok(u8::from_be_bytes(result.try_into().expect("Invalid read")))
    }

    fn read_uint16(&mut self) -> Result<u16, XRPLBinaryCodecException> {
        let result = self.read(2)?;
        Ok(u16::from_be_bytes(result.try_into().expect("Invalid read")))
    }

    fn read_uint32(&mut self) -> Result<u32, XRPLBinaryCodecException> {
        let result = self.read(4)?;
        Ok(u32::from_be_bytes(result.try_into().expect("Invalid read")))
    }

    fn is_end(&self, custom_end: Option<usize>) -> bool {
        if let Some(end) = custom_end {
            self.0.len() <= end
        } else {
            self.0.is_empty()
        }
    }

    fn read_length_prefix(&mut self) -> Result<usize, XRPLBinaryCodecException> {
        let byte1: usize = self.read_uint8()? as usize;

        match byte1 {
            // If the field contains 0 to 192 bytes of data,
            // the first byte defines the length of the contents.
            x if x <= MAX_SINGLE_BYTE_LENGTH => Ok(byte1),
            // If the field contains 193 to 12480 bytes of data,
            // the first two bytes indicate the length of the
            // field with the following formula:
            // 193 + ((byte1 - 193) * 256) + byte2
            x if x <= MAX_SECOND_BYTE_VALUE => {
                let byte2: usize = self.read_uint8()? as usize;
                Ok((MAX_SINGLE_BYTE_LENGTH + 1)
                    + ((byte1 - (MAX_SINGLE_BYTE_LENGTH + 1)) * MAX_BYTE_VALUE)
                    + byte2)
            }
            // If the field contains 12481 to 918744 bytes of data,
            // the first three bytes indicate the length of the
            // field with the following formula:
            // 12481 + ((byte1 - 241) * 65536) + (byte2 * 256) + byte3
            x if x <= 254 => {
                let byte2: usize = self.read_uint8()? as usize;
                let byte3: usize = self.read_uint8()? as usize;

                Ok(MAX_DOUBLE_BYTE_LENGTH
                    + ((byte1 - (MAX_SECOND_BYTE_VALUE + 1)) * MAX_DOUBLE_BYTE_VALUE)
                    + (byte2 * MAX_BYTE_VALUE)
                    + byte3)
            }
            _ => Err(XRPLBinaryCodecException::new(
                "Length prefix must contain between 1 and 3 bytes.",
            )),
        }
    }

    fn read_field_header(&mut self) -> Result<FieldHeader, XRPLBinaryCodecException> {
        let mut type_code: i16 = self.read_uint8()? as i16;
        let mut field_code: i16 = type_code & 15;

        type_code >>= 4;

        if type_code == 0 {
            type_code = self.read_uint8()? as i16;

            if type_code == 0 || type_code < 16 {
                return Err(XRPLBinaryCodecException::new(
                    "Cannot read field ID, type_code out of range.",
                ));
            };
        };

        if field_code == 0 {
            field_code = self.read_uint8()? as i16;

            if field_code == 0 || field_code < 16 {
                return Err(XRPLBinaryCodecException::new(
                    "Cannot read field ID, field_code out of range.",
                ));
            };
        };

        Ok(FieldHeader {
            type_code,
            field_code,
        })
    }

    fn read_field(&mut self) -> Result<FieldInstance, XRPLBinaryCodecException> {
        let field_header = self.read_field_header()?;
        let field_name = get_field_name_from_header(&field_header);

        if let Some(name) = field_name {
            if let Some(instance) = get_field_instance(name) {
                return Ok(instance);
            };
        };

        Err(XRPLBinaryCodecException::new("No such field name"))
    }
}

impl From<&[u8]> for BinaryParser {
    fn from(hex_bytes: &[u8]) -> Self {
        BinaryParser(hex_bytes.to_vec())
    }
}

impl PartialEq<[u8]> for BinaryParser {
    fn eq(&self, bytes: &[u8]) -> bool {
        self.0 == bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_HEX: &str = "00112233445566";

    #[test]
    fn test_peek() {
        let test_bytes: Vec<u8> = hex::decode(TEST_HEX).unwrap();
        let binary_parser = BinaryParser::from(test_bytes.as_ref());
        let first_byte = binary_parser.peek().unwrap();

        assert_eq!([test_bytes[0]; 1], first_byte);
    }

    #[test]
    fn test_skip() {
        let test_bytes: Vec<u8> = hex::decode(TEST_HEX).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());

        assert!(binary_parser.skip(4).is_ok());
        assert_eq!(binary_parser, test_bytes[4..]);
    }

    #[test]
    fn test_read() {
        let test_bytes: Vec<u8> = hex::decode(TEST_HEX).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let result = binary_parser.read(5);

        assert!(result.is_ok());
        assert_eq!(test_bytes[..5], result.unwrap());
    }

    #[test]
    fn test_read_uint8() {
        let test_hex: &str = "01000200000003";
        let test_bytes: Vec<u8> = hex::decode(test_hex).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let result = binary_parser.read_uint8();

        assert!(result.is_ok());
        assert_eq!(1, result.unwrap());
    }

    #[test]
    fn test_read_uint16() {
        let test_hex: &str = "000200000003";
        let test_bytes: Vec<u8> = hex::decode(test_hex).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let result = binary_parser.read_uint16();

        assert!(result.is_ok());
        assert_eq!(2, result.unwrap());
    }

    #[test]
    fn test_read_uint32() {
        let test_hex: &str = "00000003";
        let test_bytes: Vec<u8> = hex::decode(test_hex).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let result = binary_parser.read_uint32();

        assert!(result.is_ok());
        assert_eq!(3, result.unwrap());
    }

    #[test]
    fn test_read_length_prefix() {
        let test_bytes: Vec<u8> = hex::decode(TEST_HEX).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let result = binary_parser.read_length_prefix();

        assert!(result.is_ok());
        assert_eq!(0, result.unwrap());
    }

    #[test]
    fn accept_peek_skip_read() {
        let test_bytes: Vec<u8> = hex::decode(TEST_HEX).unwrap();
        let mut binary_parser = BinaryParser::from(test_bytes.as_ref());
        let first_byte = binary_parser.peek().unwrap();

        assert_eq!([test_bytes[0]; 1], first_byte);
        assert!(binary_parser.skip(3).is_ok());
        assert_eq!(binary_parser, test_bytes[3..]);

        let result = binary_parser.read(2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_bytes[3..5]);
    }
}
