//! Codec for serializing and deserializing Amount fields.
//!
//! See Amount Fields:
//! `<https://xrpl.org/serialization.html#amount-fields>`

use crate::core::binarycodec::types::xrpl_type::Buffered;
use crate::core::binarycodec::types::xrpl_type::XRPLType;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::Deserialize;

const _MIN_MANTISSA: u64 = u64::pow(10, 15);
const _MAX_MANTISSA: u64 = u64::pow(10, 16) - 1;

const _NOT_XRP_BIT_MASK: u8 = 0x80;
const _POS_SIGN_BIT_MASK: u64 = 0x4000000000000000;
const _ZERO_CURRENCY_AMOUNT_HEX: u64 = 0x8000000000000000;
const _NATIVE_AMOUNT_BYTE_LENGTH: u8 = 8;
const _CURRENCY_AMOUNT_BYTE_LENGTH: u8 = 48;

/// Returns True if the given string contains a
/// decimal point character.
fn _contains_decimal(string: &str) -> bool {
    string.contains(".")
}

/// Codec for serializing and deserializing Amount fields.
///
/// See Amount Fields:
/// `<https://xrpl.org/serialization.html#amount-fields>`
#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "&str")]
pub struct Amount(Vec<u8>);

impl XRPLType for Amount {
    type Error = hex::FromHexError;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Amount(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl Buffered for Amount {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&str> for Amount {
    type Error = hex::FromHexError;

    /// Construct a Hash object from a hex string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Amount::new(Some(&hex::decode(value)?))?)
    }
}
