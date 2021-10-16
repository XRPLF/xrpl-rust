//! Codec for serializing and deserializing a hash
//! field with a width of 128 bits (16 bytes).
//!
//! See Hash Fields:
//! `<https://xrpl.org/serialization.html#hash-fields>`

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::types::hash::Hash;
use crate::core::binarycodec::types::serialized_type::Buffered;
use crate::core::binarycodec::types::serialized_type::Serializable;
use alloc::vec::Vec;
use core::convert::TryFrom;

/// Codec for serializing and deserializing a hash field
/// with a width of 128 bits (16 bytes).
///
/// See Hash Fields:
/// `<https://xrpl.org/serialization.html#hash-fields>`
pub struct Hash128(Vec<u8>);

impl Hash for Hash128 {}

impl TryFrom<&str> for Hash128 {
    type Error = hex::FromHexError;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Hash128(hex::decode(value)?))
    }
}

impl Serializable for Hash128 {
    fn new(bytes: Option<&[u8]>) -> Self {
        Hash128(<dyn Hash>::make(bytes))
    }

    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<Hash128, XRPLBinaryCodecException> {
        Ok(Hash128(<dyn Hash>::parse(
            parser,
            length.or(Some(16)).unwrap(),
        )?))
    }
}

impl Buffered for Hash128 {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}
