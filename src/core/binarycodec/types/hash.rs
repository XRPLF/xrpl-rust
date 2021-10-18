//! Base class for XRPL Hash types.
//!
//! See Hash Fields:
//! `<https://xrpl.org/serialization.html#hash-fields>`

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::binary_wrappers::binary_parser::Parser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::vec::Vec;

/// XRPL Hash type.
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
pub(crate) trait Hash {
    fn get_length() -> usize
    where
        Self: Sized;
}

impl dyn Hash {
    pub fn make<T: Hash>(bytes: Option<&[u8]>) -> Result<Vec<u8>, XRPLBinaryCodecException> {
        let byte_value: &[u8] = bytes.or(Some(&[])).unwrap();
        let hash_length: usize = T::get_length();

        if byte_value.len() != hash_length {
            Err(XRPLBinaryCodecException::InvalidHashLength {
                expected: hash_length,
                found: byte_value.len(),
            })
        } else {
            Ok(bytes.unwrap().to_vec())
        }
    }

    pub fn parse<T: Hash>(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Vec<u8>, XRPLBinaryCodecException> {
        let read_length = length.or(Some(T::get_length())).unwrap();
        parser.read(read_length)
    }
}
