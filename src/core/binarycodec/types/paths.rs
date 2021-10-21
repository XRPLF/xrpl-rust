//! Codec for serializing and deserializing PathSet fields.
//!
//! See PathSet Fields:
//! `<https://xrpl.org/serialization.html#pathset-fields>`

use crate::alloc::string::ToString;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::binary_wrappers::binary_parser::Parser;
use crate::core::binarycodec::types::account_id::AccountId;
use crate::core::binarycodec::types::currency::Currency;
use crate::core::binarycodec::types::utils::ACCOUNT_ID_LENGTH;
use crate::core::binarycodec::types::utils::CURRENCY_CODE_LENGTH;
use crate::core::binarycodec::types::xrpl_type::Buffered;
use crate::core::binarycodec::types::xrpl_type::FromParser;
use crate::core::binarycodec::types::xrpl_type::XRPLType;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use indexmap::IndexMap;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Constant for masking types of a PathStep
const _TYPE_ACCOUNT: u8 = 0x01;
const _TYPE_CURRENCY: u8 = 0x10;
const _TYPE_ISSUER: u8 = 0x20;

// Constants for separating Paths in a PathSet
const _PATHSET_END_BYTE: u8 = 0x00;
const _PATH_SEPARATOR_BYTE: u8 = 0xFF;

/// Helper function to determine if a dictionary represents
/// a valid path step.
fn _is_path_step(value: &IndexMap<String, String>) -> bool {
    value.contains_key("issuer") || value.contains_key("account") || value.contains_key("currency")
}

/// Helper function to determine if a list represents a
/// valid path set.
fn _is_path_set(value: Vec<Vec<IndexMap<String, String>>>) -> bool {
    value.len() == 0 || value[0].len() == 0 || _is_path_step(&value[0][0])
}

#[derive(Debug, Clone)]
pub struct PathStep(Vec<u8>);

impl XRPLType for PathStep {
    type Error = XRPLAddressCodecException;

    /// Construct an AccountID from given bytes.
    /// If buffer is not provided, default to 20 zero bytes.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(PathStep(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl Buffered for PathStep {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl FromParser for PathStep {
    type Error = XRPLAddressCodecException;

    /// Construct a PathStep object from an existing BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        length: Option<usize>,
    ) -> Result<PathStep, Self::Error> {
        let value_bytes: Vec<u8> = vec![];
        let data_type = parser.read_uint8()?;
        let mut buffer = vec![];

        if data_type & _TYPE_ACCOUNT != 0 {
            value_bytes.extend_from_slice(&parser.read(ACCOUNT_ID_LENGTH)?);
        };

        if data_type & _TYPE_CURRENCY != 0 {
            value_bytes.extend_from_slice(&parser.read(CURRENCY_CODE_LENGTH)?);
        };

        if data_type & _TYPE_ISSUER != 0 {
            value_bytes.extend_from_slice(&parser.read(ACCOUNT_ID_LENGTH)?);
        };

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        PathStep::new(Some(&buffer))
    }
}

impl Serialize for PathStep {
    /// Returns the JSON representation of a PathStep.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = vec![];
        let mut parser = BinaryParser::from(self.get_buffer());
        let data_type = parser.read_uint8()?;
        let account: Option<String>;
        let currency: Option<String>;
        let issuer: Option<String>;

        if data_type & _TYPE_ACCOUNT != 0 {
            account = Some(AccountId::from_parser(&mut parser.clone(), None)?.to_string());
        } else {
            account = None;
        }

        if data_type & _TYPE_CURRENCY != 0 {
            currency = Some(Currency::from_parser(&mut parser.clone(), None)?.to_string());
        } else {
            currency = None;
        }

        if data_type & _TYPE_ISSUER != 0 {
            issuer = Some(AccountId::from_parser(&mut parser.clone(), None)?.to_string());
        } else {
            issuer = None;
        }

        json!({
            "account": account,
            "currency": currency,
            "issuer": issuer,
        })
    }
}

impl TryFrom<&IndexMap<String, String>> for PathStep {
    type Error = XRPLAddressCodecException;

    /// Construct a PathStep object from a dictionary.
    fn try_from(value: &IndexMap<String, String>) -> Result<Self, Self::Error> {
        let value_bytes: Vec<u8> = vec![];
        let mut data_type = 0x00;
        let mut buffer = vec![];

        if value.contains_key("account") {
            let data = AccountId::try_from(value["account"].as_ref())?.get_buffer();
            data_type |= _TYPE_ACCOUNT;

            value_bytes.extend_from_slice(data);
        };

        if value.contains_key("currency") {
            let data = Currency::try_from(value["currency"].as_ref())?.get_buffer();
            data_type |= _TYPE_CURRENCY;

            value_bytes.extend_from_slice(data);
        };

        if value.contains_key("issuer") {
            let data = AccountId::try_from(value["issuer"].as_ref())?.get_buffer();
            data_type |= _TYPE_ISSUER;

            value_bytes.extend_from_slice(data);
        };

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        Self::new(Some(&buffer))
    }
}
