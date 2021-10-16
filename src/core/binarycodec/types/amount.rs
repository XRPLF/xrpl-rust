//! Codec for serializing and deserializing Amount fields.
//!
//! See Amount Fields:
//! `<https://xrpl.org/serialization.html#amount-fields>`

use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::types::hash::Hash;
use crate::core::binarycodec::types::serialized_type::Buffered;
use crate::core::binarycodec::types::serialized_type::Serializable;
use alloc::vec::Vec;
use anyhow::Result;
use core::convert::TryFrom;

/// Codec for serializing and deserializing Amount fields.
///
/// See Amount Fields:
/// `<https://xrpl.org/serialization.html#amount-fields>`
pub struct Amount {
    bytes: Vec<u8>,
    pub value: String,
    pub currency: Currency,
    pub issuer: Issuer,
}

impl TryFrom<&str> for Amount {
    type Error = hex::FromHexError;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Amount(hex::decode(value)?))
    }
}

impl Serializable for Amount {
    fn new(bytes: &[u8]) -> Self {
        Amount(bytes.to_vec())
    }

    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Amount> {
        let parser_first_byte = parser.peek();
    }
}

impl Buffered for Amount {
    fn get_buffer(&self) -> &[u8] {
        &self.bytes
    }
}
