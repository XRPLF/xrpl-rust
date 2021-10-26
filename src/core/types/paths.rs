//! Codec for serializing and deserializing PathSet fields.
//!
//! See PathSet Fields:
//! `<https://xrpl.org/serialization.html#pathset-fields>`

use crate::constants::ACCOUNT_ID_LENGTH;
use crate::core::addresscodec::exceptions::XRPLAddressCodecException;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::BinaryParser;
use crate::core::binarycodec::Parser;
use crate::core::types::account_id::AccountId;
use crate::core::types::currency::Currency;
use crate::core::types::utils::CURRENCY_CODE_LENGTH;
use crate::core::types::*;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use indexmap::IndexMap;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::Serializer;
use serde::{Deserialize, Serialize};

// Constant Keys
const _ACC_KEY: &str = "account";
const _CUR_KEY: &str = "currency";
const _ISS_KEY: &str = "issuer";

// Constant for masking types of a PathStep
const _TYPE_ACCOUNT: u8 = 0x01;
const _TYPE_CURRENCY: u8 = 0x10;
const _TYPE_ISSUER: u8 = 0x20;

// Constants for separating Paths in a PathSet
const _PATHSET_END_BYTE: u8 = 0x00;
const _PATH_SEPARATOR_BYTE: u8 = 0xFF;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PathStepData {
    #[serde(skip_serializing)]
    index: u8,
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
    value.contains_key(_ISS_KEY) || value.contains_key(_ACC_KEY) || value.contains_key(_CUR_KEY)
}

/// Helper function to determine if a list represents a
/// valid path set.
fn _is_path_set(value: &[Vec<IndexMap<String, String>>]) -> bool {
    value.is_empty() || value[0].is_empty() || _is_path_step(&value[0][0])
}

impl XRPLType for PathStep {
    type Error = XRPLBinaryCodecException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(PathStep(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl XRPLType for Path {
    type Error = XRPLBinaryCodecException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(Path(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl XRPLType for PathSet {
    type Error = XRPLBinaryCodecException;

    fn new(buffer: Option<&[u8]>) -> Result<Self, Self::Error> {
        Ok(PathSet(buffer.or_else(|| Some(&[])).unwrap().to_vec()))
    }
}

impl PathStepData {
    /// Constract a new instance of PathStepData.
    pub fn new(account: Option<String>, currency: Option<String>, issuer: Option<String>) -> Self {
        Self {
            index: _PATHSET_END_BYTE | _TYPE_ACCOUNT,
            account,
            currency,
            issuer,
        }
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

    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> Result<PathStepData, Self::Error> {
        let account: Option<String>;
        let currency: Option<String>;
        let issuer: Option<String>;
        let data_type = parser.read_uint8()?;

        if data_type & _TYPE_ACCOUNT != 0 {
            let data = AccountId::from_parser(parser, None)?;
            account = Some(data.to_string());
        } else {
            account = None;
        }

        if data_type & _TYPE_CURRENCY != 0 {
            let data = Currency::from_parser(parser, None)?;
            currency = Some(data.to_string());
        } else {
            currency = None;
        }

        if data_type & _TYPE_ISSUER != 0 {
            let data = AccountId::from_parser(parser, None)?;
            issuer = Some(data.to_string());
        } else {
            issuer = None;
        }

        Ok(PathStepData::new(account, currency, issuer))
    }
}

impl FromParser for Path {
    type Error = XRPLBinaryCodecException;

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

        if let Ok(pathdata) = result {
            let mut builder = serializer.serialize_map(None)?;

            for path in pathdata {
                let (key, value) = path;
                if let Some(data) = value {
                    builder.serialize_entry(&key, &data)?;
                } else {
                    continue;
                }
            }

            builder.end()
        } else {
            Err(serde::ser::Error::custom(result.unwrap_err()))
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
                return Err(serde::ser::Error::custom(pathstep.unwrap_err()));
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
        let mut sequence = serializer.serialize_seq(None)?;
        let mut parser = BinaryParser::from(self.get_buffer());

        while !parser.is_end(None) {
            let path = Path::from_parser(&mut parser, None);

            if let Ok(step) = path {
                sequence.serialize_element(&step)?;

                if let Err(err) = parser.skip_bytes(1) {
                    return Err(serde::ser::Error::custom(err));
                };
            } else {
                return Err(serde::ser::Error::custom(path.unwrap_err()));
            }
        }

        sequence.end()
    }
}

impl TryFrom<IndexMap<String, String>> for PathStep {
    type Error = XRPLAddressCodecException;

    /// Construct a PathStep object from a dictionary.
    fn try_from(value: IndexMap<String, String>) -> Result<Self, Self::Error> {
        let mut value_bytes: Vec<u8> = vec![];
        let mut data_type = 0x00;
        let mut buffer = vec![];

        if value.contains_key(_ACC_KEY) {
            let data = AccountId::try_from(value[_ACC_KEY].as_ref())?;
            data_type |= _TYPE_ACCOUNT;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        if value.contains_key(_CUR_KEY) {
            let data = Currency::try_from(value[_CUR_KEY].as_ref())?;
            data_type |= _TYPE_CURRENCY;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        if value.contains_key(_ISS_KEY) {
            let data = AccountId::try_from(value[_ISS_KEY].as_ref())?;
            data_type |= _TYPE_ISSUER;

            value_bytes.extend_from_slice(data.get_buffer());
        };

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        Ok(Self::new(Some(&buffer))?)
    }
}

impl TryFrom<Vec<IndexMap<String, String>>> for Path {
    type Error = XRPLAddressCodecException;

    /// Construct a Path object from a list.
    fn try_from(value: Vec<IndexMap<String, String>>) -> Result<Self, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        for step in value {
            let pathstep = PathStep::try_from(step)?;
            buffer.extend_from_slice(pathstep.get_buffer());
        }

        Ok(Path::new(Some(&buffer))?)
    }
}

impl TryFrom<Vec<Vec<IndexMap<String, String>>>> for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathSet object from a list.
    fn try_from(value: Vec<Vec<IndexMap<String, String>>>) -> Result<Self, Self::Error> {
        if _is_path_set(&value) {
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

impl TryFrom<&str> for Path {
    type Error = XRPLAddressCodecException;

    /// Construct a Path object from a string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let json: Vec<IndexMap<String, String>> = serde_json::from_str(value)?;
        Self::try_from(json)
    }
}

impl TryFrom<&str> for PathSet {
    type Error = XRPLBinaryCodecException;

    /// Construct a PathSet object from a string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let json: Vec<Vec<IndexMap<String, String>>> = serde_json::from_str(value)?;
        Self::try_from(json)
    }
}

impl Iterator for PathStepData {
    type Item = (String, Option<String>);

    fn next(&mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.index & _TYPE_ACCOUNT != 0 {
            self.index = _TYPE_CURRENCY;
            Some((_ACC_KEY.to_string(), self.account.to_owned()))
        } else if self.index & _TYPE_CURRENCY != 0 {
            self.index = _TYPE_ISSUER;
            Some((_CUR_KEY.to_string(), self.currency.to_owned()))
        } else if self.index & _TYPE_ISSUER != 0 {
            self.index = _PATHSET_END_BYTE;
            Some((_ISS_KEY.to_string(), self.issuer.to_owned()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::types::test_cases::TEST_PATH_BUFFER;
    use crate::core::types::test_cases::TEST_PATH_SET_BUFFER;
    use crate::core::types::test_cases::TEST_PATH_STEP_BUFFER;

    pub const PATH_SET_TEST: &str = include_str!("../test_data/path-set-test.json");
    pub const PATH_TEST: &str = include_str!("../test_data/path-test.json");

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
    fn test_pathstep_new() {
        for data in TEST_PATH_STEP_BUFFER {
            let hex: Vec<u8> = hex::decode(data).unwrap();
            let pathstep = PathStep::new(Some(&hex)).unwrap();

            assert_eq!(hex, pathstep.get_buffer());
        }
    }

    #[test]
    fn test_path_new() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_BUFFER).unwrap();
        let path = Path::new(Some(&hex)).unwrap();

        assert_eq!(TEST_PATH_BUFFER, hex::encode_upper(path.get_buffer()));
    }

    #[test]
    fn test_pathset_new() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_SET_BUFFER).unwrap();
        let pathset = PathSet::new(Some(&hex)).unwrap();

        assert_eq!(
            TEST_PATH_SET_BUFFER,
            hex::encode_upper(pathset.get_buffer())
        );
    }

    #[test]
    fn test_pathstep_from_parser() {
        for data in TEST_PATH_STEP_BUFFER {
            let hex = hex::decode(data).unwrap();
            let mut parser = BinaryParser::from(hex.clone());
            let pathset = PathStep::from_parser(&mut parser, None).unwrap();

            assert_eq!(hex, pathset.get_buffer());
        }
    }

    #[test]
    fn test_path_from_parser() {
        let hex = hex::decode(TEST_PATH_BUFFER).unwrap();
        let mut parser = BinaryParser::from(hex.clone());
        let pathset = Path::from_parser(&mut parser, None).unwrap();

        assert_eq!(hex, pathset.get_buffer());
    }

    #[test]
    fn test_pathset_from_parser() {
        let hex = hex::decode(TEST_PATH_SET_BUFFER).unwrap();
        let mut parser = BinaryParser::from(hex.clone());
        let pathset = PathSet::from_parser(&mut parser, None).unwrap();

        assert_eq!(hex, pathset.get_buffer());
    }

    #[test]
    fn test_pathstep_try_from() {
        let json: Vec<IndexMap<String, String>> = serde_json::from_str(PATH_TEST).unwrap();
        let mut pathsteps: Vec<u8> = vec![];

        for map in json {
            pathsteps.extend_from_slice(PathStep::try_from(map.clone()).unwrap().get_buffer());
        }

        assert_eq!(TEST_PATH_BUFFER, hex::encode_upper(pathsteps));
    }

    #[test]
    fn test_path_try_from() {
        let hex = hex::decode(TEST_PATH_BUFFER).unwrap();
        let path = Path::try_from(PATH_TEST).unwrap();

        assert_eq!(hex, path.get_buffer())
    }

    #[test]
    fn test_pathset_try_from() {
        let hex = hex::decode(TEST_PATH_SET_BUFFER).unwrap();
        let pathset = PathSet::try_from(PATH_SET_TEST).unwrap();

        assert_eq!(hex, pathset.get_buffer())
    }

    #[test]
    fn test_pathstep_to_json() {
        let json: Vec<IndexMap<String, String>> = serde_json::from_str(PATH_TEST).unwrap();

        for map in json {
            let pathstep = PathStep::try_from(map.clone()).unwrap();
            assert_eq!(
                serde_json::to_string(&map).unwrap(),
                serde_json::to_string(&pathstep).unwrap()
            );
        }
    }

    #[test]
    fn test_path_to_json() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_BUFFER).unwrap();
        let path = Path::new(Some(&hex)).unwrap();
        let compact: serde_json::Value = serde_json::from_str(PATH_TEST).unwrap();

        assert_eq!(
            serde_json::to_string(&compact).unwrap(),
            serde_json::to_string(&path).unwrap()
        );
    }

    #[test]
    fn test_pathset_to_json() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_SET_BUFFER).unwrap();
        let pathset = PathSet::new(Some(&hex)).unwrap();
        let compact: serde_json::Value = serde_json::from_str(PATH_SET_TEST).unwrap();

        assert_eq!(
            serde_json::to_string(&compact).unwrap(),
            serde_json::to_string(&pathset).unwrap()
        );
    }
}
