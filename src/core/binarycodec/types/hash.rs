//! Base class for XRPL Hash types.
//! See Hash Fields:
//! `<https://xrpl.org/serialization.html#hash-fields>`

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::binary_wrappers::binary_parser::Parser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use alloc::vec::Vec;

/// XRPL Hash type.
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
pub(crate) trait Hash {}

impl dyn Hash {
    pub(crate) fn make(bytes: Option<&[u8]>) -> Vec<u8> {
        bytes.or(Some(&[])).unwrap().into()
    }

    pub(crate) fn parse(
        parser: &mut BinaryParser,
        length: usize,
    ) -> Result<Vec<u8>, XRPLBinaryCodecException> {
        parser.read(length)
    }
}
