//! Codec for serializing and deserializing PathSet fields.
//!
//! See PathSet Fields:
//! `<https://xrpl.org/serialization.html#pathset-fields>`

use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::binarycodec::binary_wrappers::binary_parser::BinaryParser;
use crate::core::binarycodec::binary_wrappers::binary_parser::Parser;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
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
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::Serializer;
use serde::{Deserialize, Serialize};

// Constant for masking types of a PathStep
const _TYPE_ACCOUNT: u8 = 0x01;
const _TYPE_CURRENCY: u8 = 0x10;
const _TYPE_ISSUER: u8 = 0x20;

// Constants for separating Paths in a PathSet
const _PATHSET_END_BYTE: u8 = 0x00;
const _PATH_SEPARATOR_BYTE: u8 = 0xFF;

/// JSON serialization error.
const JSON_ERROR: &str = "JSON Serialization failed.";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PathStepData {
    #[serde(skip_serializing_if = "Option::is_none")]
    account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,
}

/// Serialize and deserialize a single step in a Path.
#[derive(Debug, Clone)]
pub struct PathStep(Vec<u8>);

/// Class for serializing/deserializing Paths.
#[derive(Debug, Clone)]
pub struct Path(Vec<u8>);

/// Class for serializing/deserializing Paths.
#[derive(Debug, Clone)]
pub struct PathSet(Vec<u8>);

/// Helper function to determine if a dictionary represents
/// a valid path step.
fn _is_path_step(value: &IndexMap<String, String>) -> bool {
    value.contains_key("issuer") || value.contains_key("account") || value.contains_key("currency")
}

/// Helper function to determine if a list represents a
/// valid path set.
fn _is_path_set(value: &[Vec<IndexMap<String, String>>]) -> bool {
    value.is_empty() || value[0].is_empty() || _is_path_step(&value[0][0])
}

impl XRPLType for PathStep {
    type Error = XRPLBinaryCodecException;

    /// Construct an PathStep from given bytes.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(PathStep(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl XRPLType for Path {
    type Error = XRPLBinaryCodecException;

    /// Construct an Path from given bytes.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Path(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl XRPLType for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct an PathSet from given bytes.
    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(PathSet(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl Buffered for PathStep {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl Buffered for Path {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl Buffered for PathSet {
    fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}

impl FromParser for PathStep {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathStep object from an existing BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> Result<PathStep, Self::Error> {
        let mut value_bytes: Vec<u8> = vec![];
        let mut buffer: Vec<u8> = vec![];
        let data_type = parser.read_uint8()?;

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

impl FromParser for PathStepData {
    type Error = XRPLBinaryCodecException;

    /// ConstructStepData a Path object from an existing BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> Result<PathStepData, Self::Error> {
        let account: Option<String>;
        let currency: Option<String>;
        let issuer: Option<String>;
        let data_type = parser.read_uint8()?;

        if data_type & _TYPE_ACCOUNT != 0 {
            let data = AccountId::from_parser(&mut parser.clone(), None)?;
            account = Some(serde_json::to_string(&data)?);
        } else {
            account = None;
        }

        if data_type & _TYPE_CURRENCY != 0 {
            let data = Currency::from_parser(&mut parser.clone(), None)?;
            currency = Some(serde_json::to_string(&data)?);
        } else {
            currency = None;
        }

        if data_type & _TYPE_ISSUER != 0 {
            let data = AccountId::from_parser(&mut parser.clone(), None)?;
            issuer = Some(serde_json::to_string(&data)?);
        } else {
            issuer = None;
        }

        Ok(PathStepData {
            account,
            currency,
            issuer,
        })
    }
}

impl FromParser for Path {
    type Error = XRPLBinaryCodecException;

    /// Construct a Path object from an existing BinaryParser.
    fn from_parser(parser: &mut BinaryParser, _length: Option<usize>) -> Result<Path, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        while !parser.is_end(None) {
            let pathstep = PathStep::from_parser(parser, None)?;
            buffer.extend_from_slice(pathstep.get_buffer());

            if parser.peek() == Some([_PATHSET_END_BYTE; 1])
                || parser.peek() == Some([_PATH_SEPARATOR_BYTE; 1])
            {
                break;
            }
        }

        Path::new(Some(&buffer))
    }
}

impl FromParser for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathSet object from an existing BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> Result<PathSet, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        while !parser.is_end(None) {
            let path = Path::from_parser(parser, None)?;

            buffer.extend_from_slice(path.get_buffer());
            buffer.extend_from_slice(&parser.read(1)?);

            let len = buffer.len();

            if buffer[len - 1] == _PATHSET_END_BYTE {
                break;
            }
        }

        PathSet::new(Some(&buffer))
    }
}

impl Serialize for PathStep {
    /// Returns the JSON representation of a PathStep.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut parser = BinaryParser::from(self.get_buffer());
        let result = PathStepData::from_parser(&mut parser, None);

        if let Ok(path) = result {
            let mut builder = serializer.serialize_struct("Path", 3)?;

            builder.serialize_field("account", &Some(path.account))?;
            builder.serialize_field("currency", &Some(path.currency))?;
            builder.serialize_field("issuer", &Some(path.issuer))?;

            builder.end()
        } else {
            Err(serde::ser::Error::custom(JSON_ERROR))
        }
    }
}

impl Serialize for Path {
    /// Returns the JSON representation of a Path.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut parser = BinaryParser::from(self.get_buffer());

        while !parser.is_end(None) {
            let pathstep = PathStep::from_parser(&mut parser, None);

            if let Ok(step) = pathstep {
                sequence.serialize_element(&step)?;
            } else {
                return Err(serde::ser::Error::custom(JSON_ERROR));
            }
        }

        sequence.end()
    }
}

impl Serialize for PathSet {
    /// Returns the JSON representation of a Path.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut catcher: Result<&BinaryParser, XRPLBinaryCodecException>;
        let mut sequence = serializer.serialize_seq(None)?;
        let mut parser = BinaryParser::from(self.get_buffer());

        while !parser.is_end(None) {
            let path = Path::from_parser(&mut parser, None);

            if let Ok(step) = path {
                sequence.serialize_element(&step)?;
                catcher = parser.skip_bytes(1);
            } else {
                return Err(serde::ser::Error::custom(JSON_ERROR));
            }

            if catcher.is_err() {
                return Err(serde::ser::Error::custom(JSON_ERROR));
            }
        }

        sequence.end()
    }
}

impl TryFrom<&IndexMap<String, String>> for PathStep {
    type Error = XRPLAddressCodecException;

    /// Construct a PathStep object from a dictionary.
    fn try_from(value: &IndexMap<String, String>) -> Result<Self, Self::Error> {
        let mut value_bytes: Vec<u8> = vec![];
        let mut data_type = 0x00;
        let mut buffer = vec![];

        if value.contains_key("account") {
            let data = AccountId::try_from(value["account"].as_ref())?;
            data_type |= _TYPE_ACCOUNT;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        if value.contains_key("currency") {
            let data = Currency::try_from(value["currency"].as_ref())?;
            data_type |= _TYPE_CURRENCY;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        if value.contains_key("issuer") {
            let data = AccountId::try_from(value["issuer"].as_ref())?;
            data_type |= _TYPE_ISSUER;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        Ok(Self::new(Some(&buffer))?)
    }
}

impl TryFrom<&Vec<IndexMap<String, String>>> for Path {
    type Error = XRPLAddressCodecException;

    /// Construct a Path object from a list.
    fn try_from(value: &Vec<IndexMap<String, String>>) -> Result<Self, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        for step in value {
            let pathstep = PathStep::try_from(step)?;
            buffer.extend_from_slice(pathstep.get_buffer());
        }

        Ok(Path::new(Some(&buffer))?)
    }
}

impl TryFrom<&Vec<Vec<IndexMap<String, String>>>> for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathSet object from a list.
    fn try_from(value: &Vec<Vec<IndexMap<String, String>>>) -> Result<Self, Self::Error> {
        if _is_path_set(value) {
            let mut buffer: Vec<u8> = vec![];

            for path_val in value {
                let result = Path::try_from(path_val);

                if let Ok(path) = result {
                    buffer.extend_from_slice(path.get_buffer());
                    buffer.extend_from_slice(&[_PATH_SEPARATOR_BYTE; 1]);
                } else {
                    return Err(XRPLBinaryCodecException::InvalidPathSetFromValue);
                }
            }

            let len = buffer.len();
            buffer[len - 1] = _PATHSET_END_BYTE;

            PathSet::new(Some(&buffer))
        } else {
            Err(XRPLBinaryCodecException::InvalidPathSetFromValue)
        }
    }
}

impl TryFrom<&str> for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathSet object from a string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let json: Vec<Vec<IndexMap<String, String>>> = serde_json::from_str(value)?;
        Self::try_from(&json)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::binarycodec::types::test_cases::TEST_PATH_SET_BUFFER;
    extern crate std;
    //std::println!("{:?}", json);

    pub const PATH_SET_TEST: &str = include_str!("../test_data/path-set-test.json");

    #[test]
    fn test_is_path_step() {
        let json: Vec<Vec<IndexMap<String, String>>> = serde_json::from_str(PATH_SET_TEST).unwrap();
        assert!(_is_path_set(&json));
    }

    #[test]
    fn test_is_path_set() {
        let json: Vec<Vec<IndexMap<String, String>>> = serde_json::from_str(PATH_SET_TEST).unwrap();

        for path in json {
            for step in path {
                assert!(_is_path_step(&step));
            }
        }
    }

    #[test]
    fn test_pathstep_new() {}

    #[test]
    fn test_path_new() {}

    #[test]
    fn test_pathset_new() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_SET_BUFFER).unwrap();
        let pathstep = PathSet::new(Some(&hex)).unwrap();

        assert_eq!(
            TEST_PATH_SET_BUFFER,
            hex::encode_upper(pathstep.get_buffer())
        );
    }

    #[test]
    fn test_pathstep_from_parser() {}

    #[test]
    fn test_path_from_parser() {}

    #[test]
    fn test_pathset_from_parser() {
        // let compact: serde_json::Value = serde_json::from_str(PATH_SET_TEST).unwrap();
        // let mut parser = BinaryParser::from(hex::decode(TEST_PATH_SET_BUFFER).unwrap());
        // let pathset = PathSet::from_parser(&mut parser, None).unwrap();

        // assert_eq!(
        //     serde_json::to_string(&compact).unwrap(),
        //     serde_json::to_string(&pathset).unwrap()
        // );
    }

    #[test]
    fn test_pathstep_try_from() {}

    #[test]
    fn test_path_try_from() {}

    #[test]
    fn test_pathset_try_from() {}

    #[test]
    fn test_pathstep_to_json() {}

    #[test]
    fn test_path_to_json() {}

    #[test]
    fn test_pathset_to_json() {}
}
