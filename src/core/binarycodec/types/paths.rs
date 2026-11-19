//! Codec for serializing and deserializing PathSet fields.
//!
//! See PathSet Fields:
//! `<https://xrpl.org/serialization.html#pathset-fields>`

use crate::constants::ACCOUNT_ID_LENGTH;
use crate::core::binarycodec::exceptions::XRPLBinaryCodecException;
use crate::core::binarycodec::types::utils::CURRENCY_CODE_LENGTH;
use crate::core::binarycodec::types::utils::HASH192_LENGTH;
use crate::core::binarycodec::types::*;
use crate::core::exceptions::XRPLCoreException;
use crate::core::exceptions::XRPLCoreResult;
use crate::core::BinaryParser;
use crate::core::Parser;
use crate::XRPLSerdeJsonError;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use indexmap::IndexMap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;

// Constant Keys
const _ACC_KEY: &str = "account";
const _CUR_KEY: &str = "currency";
const _ISS_KEY: &str = "issuer";
const _MPT_KEY: &str = "mpt_issuance_id";

// Constant for masking types of a PathStep
const _TYPE_ACCOUNT: u8 = 0x01;
const _TYPE_CURRENCY: u8 = 0x10;
const _TYPE_ISSUER: u8 = 0x20;
const _TYPE_MPT: u8 = 0x40;
/// Every path step type bit `rippled` recognises (`STPathElement::TypeAll`).
const _TYPE_ALL: u8 = _TYPE_ACCOUNT | _TYPE_CURRENCY | _TYPE_ISSUER | _TYPE_MPT;

// Constants for separating Paths in a PathSet
const _PATHSET_END_BYTE: u8 = 0x00;
const _PATH_SEPARATOR_BYTE: u8 = 0xFF;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PathStepData {
    #[serde(skip_serializing)]
    index: u8,
    account: Option<String>,
    currency: Option<String>,
    mpt_issuance_id: Option<String>,
    issuer: Option<String>,
}

/// Serialize and deserialize a single step in a Path.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "&str")]
pub struct PathStep(Vec<u8>);

/// Class for serializing/deserializing Paths.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "&str")]
pub struct Path(Vec<u8>);

/// Class for serializing/deserializing Paths.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "&str")]
pub struct PathSet(Vec<u8>);

/// Helper function to determine if a dictionary represents
/// a valid path step.
fn _is_path_step(value: &IndexMap<String, String>) -> bool {
    value.contains_key(_ISS_KEY)
        || value.contains_key(_ACC_KEY)
        || value.contains_key(_CUR_KEY)
        || value.contains_key(_MPT_KEY)
}

/// Helper function to determine if a list represents a
/// valid path set.
fn _is_path_set(value: &[Vec<IndexMap<String, String>>]) -> bool {
    value.is_empty() || value[0].is_empty() || _is_path_step(&value[0][0])
}

/// Reject a path step type byte `rippled` would reject, and no others, matching
/// `STPathSet`'s deserialising constructor:
///
/// * a bit outside [`_TYPE_ALL`],
/// * `Currency` combined with `MPT`, which name the same slot, and
/// * `0x00`, which names no fields and is the byte that ends a path set, so a
///   step with no fields is rejected here on the encode path rather than being
///   written as a terminator.
///
/// `Issuer` combined with `MPT` is **not** refused: `rippled` accepts a step
/// carrying both, reading the issuance id followed by the issuer, so this codec
/// must be able to round-trip the same step.
///
/// On the decode path terminator bytes never reach here: [`Path::from_parser`]
/// stops on `0x00` and `0xFF` before reading a step, as `rippled` checks
/// `TypeNone` and `TypeBoundary` ahead of its bad-bits test. The `0x00` check
/// below is for the encode path, which assembles the byte from the keys present.
fn _validate_path_step_type(data_type: u8) -> XRPLCoreResult<()> {
    if data_type == 0 {
        return Err(XRPLBinaryCodecException::EmptyPathStep.into());
    }

    if data_type & !_TYPE_ALL != 0 {
        return Err(XRPLBinaryCodecException::UnknownPathStepType {
            type_byte: data_type,
        }
        .into());
    }

    if data_type & _TYPE_CURRENCY != 0 && data_type & _TYPE_MPT != 0 {
        return Err(XRPLBinaryCodecException::MutuallyExclusivePathStepFields {
            type_byte: data_type,
            first: _CUR_KEY,
            second: _MPT_KEY,
        }
        .into());
    }

    Ok(())
}

impl XRPLType for PathStep {
    type Error = XRPLCoreException;

    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(PathStep(data.to_vec()))
        } else {
            Ok(PathStep(vec![]))
        }
    }
}

impl XRPLType for Path {
    type Error = XRPLCoreException;

    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(Path(data.to_vec()))
        } else {
            Ok(Path(vec![]))
        }
    }
}

impl XRPLType for PathSet {
    type Error = XRPLCoreException;

    fn new(buffer: Option<&[u8]>) -> XRPLCoreResult<Self, Self::Error> {
        if let Some(data) = buffer {
            Ok(PathSet(data.to_vec()))
        } else {
            Ok(PathSet(vec![]))
        }
    }
}

impl PathStepData {
    /// Constract a new instance of PathStepData.
    pub fn new(
        account: Option<String>,
        currency: Option<String>,
        mpt_issuance_id: Option<String>,
        issuer: Option<String>,
    ) -> Self {
        Self {
            index: _PATHSET_END_BYTE | _TYPE_ACCOUNT,
            account,
            currency,
            mpt_issuance_id,
            issuer,
        }
    }
}

impl TryFromParser for PathStep {
    type Error = XRPLCoreException;

    /// Build PathStep from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<PathStep, Self::Error> {
        let mut value_bytes: Vec<u8> = vec![];
        let mut buffer: Vec<u8> = vec![];
        let data_type = parser.read_uint8()?;
        _validate_path_step_type(data_type)?;

        if data_type & _TYPE_ACCOUNT != 0 {
            value_bytes.extend_from_slice(&parser.read(ACCOUNT_ID_LENGTH)?);
        };

        if data_type & _TYPE_CURRENCY != 0 {
            value_bytes.extend_from_slice(&parser.read(CURRENCY_CODE_LENGTH)?);
        };

        if data_type & _TYPE_MPT != 0 {
            value_bytes.extend_from_slice(&parser.read(HASH192_LENGTH)?);
        };

        if data_type & _TYPE_ISSUER != 0 {
            value_bytes.extend_from_slice(&parser.read(ACCOUNT_ID_LENGTH)?);
        };

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        PathStep::new(Some(&buffer))
    }
}

impl TryFromParser for PathStepData {
    type Error = XRPLCoreException;

    /// Build PathStepData from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<PathStepData, Self::Error> {
        let data_type = parser.read_uint8()?;
        _validate_path_step_type(data_type)?;

        let account: Option<String> = if data_type & _TYPE_ACCOUNT != 0 {
            let data = AccountId::from_parser(parser, None)?;
            Some(data.to_string())
        } else {
            None
        };

        let currency: Option<String> = if data_type & _TYPE_CURRENCY != 0 {
            let data = Currency::from_parser(parser, None)?;
            Some(data.to_string())
        } else {
            None
        };

        let mpt_issuance_id: Option<String> = if data_type & _TYPE_MPT != 0 {
            let data = Hash192::from_parser(parser, None)?;
            Some(data.to_string())
        } else {
            None
        };

        let issuer: Option<String> = if data_type & _TYPE_ISSUER != 0 {
            let data = AccountId::from_parser(parser, None)?;
            Some(data.to_string())
        } else {
            None
        };

        Ok(PathStepData::new(
            account,
            currency,
            mpt_issuance_id,
            issuer,
        ))
    }
}

impl TryFromParser for Path {
    type Error = XRPLCoreException;

    /// Build Path from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<Path, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        while !parser.is_end(None) {
            // Check for a terminator before reading a step, not after: a leading
            // 0x00 or 0xFF marks the end of the path, not a zero-field step.
            if parser.peek() == Some([_PATHSET_END_BYTE; 1])
                || parser.peek() == Some([_PATH_SEPARATOR_BYTE; 1])
            {
                break;
            }

            let pathstep = PathStep::from_parser(parser, None)?;
            buffer.extend_from_slice(pathstep.as_ref());
        }

        if buffer.is_empty() {
            // `rippled` throws "empty path" rather than accepting a path with no
            // steps.
            return Err(XRPLBinaryCodecException::EmptyPathInPathSet.into());
        }

        Path::new(Some(&buffer))
    }
}

impl TryFromParser for PathSet {
    type Error = XRPLCoreException;

    /// Build PathSet from a BinaryParser.
    fn from_parser(
        parser: &mut BinaryParser,
        _length: Option<usize>,
    ) -> XRPLCoreResult<PathSet, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        while !parser.is_end(None) {
            let path = Path::from_parser(parser, None)?;

            buffer.extend_from_slice(path.as_ref());
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
    fn serialize<S>(&self, serializer: S) -> XRPLCoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut parser = BinaryParser::from(self.as_ref());
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
    fn serialize<S>(&self, serializer: S) -> XRPLCoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut parser = BinaryParser::from(self.as_ref());

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
    fn serialize<S>(&self, serializer: S) -> XRPLCoreResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        let mut parser = BinaryParser::from(self.as_ref());

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
    type Error = XRPLCoreException;

    /// Construct a PathStep object from a dictionary.
    fn try_from(value: IndexMap<String, String>) -> XRPLCoreResult<Self, Self::Error> {
        let mut value_bytes: Vec<u8> = vec![];
        let mut data_type = 0x00;
        let mut buffer = vec![];

        if value.contains_key(_ACC_KEY) {
            let data = AccountId::try_from(value[_ACC_KEY].as_ref())?;
            data_type |= _TYPE_ACCOUNT;

            value_bytes.extend_from_slice(data.as_ref());
        };

        if value.contains_key(_CUR_KEY) {
            let data = Currency::try_from(value[_CUR_KEY].as_ref())?;
            data_type |= _TYPE_CURRENCY;

            value_bytes.extend_from_slice(data.as_ref());
        };

        if value.contains_key(_MPT_KEY) {
            let data = Hash192::try_from(value[_MPT_KEY].as_ref())?;
            data_type |= _TYPE_MPT;

            value_bytes.extend_from_slice(data.as_ref());
        };

        if value.contains_key(_ISS_KEY) {
            let data = AccountId::try_from(value[_ISS_KEY].as_ref())?;
            data_type |= _TYPE_ISSUER;

            value_bytes.extend_from_slice(data.as_ref());
        };

        _validate_path_step_type(data_type)?;

        buffer.extend_from_slice(&[data_type]);
        buffer.extend_from_slice(&value_bytes);

        Self::new(Some(&buffer))
    }
}

impl TryFrom<Vec<IndexMap<String, String>>> for Path {
    type Error = XRPLCoreException;

    /// Construct a Path object from a list.
    fn try_from(value: Vec<IndexMap<String, String>>) -> XRPLCoreResult<Self, Self::Error> {
        let mut buffer: Vec<u8> = vec![];

        for step in value {
            let pathstep = PathStep::try_from(step)?;
            buffer.extend_from_slice(pathstep.as_ref());
        }

        Path::new(Some(&buffer))
    }
}

impl TryFrom<Vec<Vec<IndexMap<String, String>>>> for PathSet {
    type Error = XRPLCoreException;

    /// Construct a PathSet object from a list.
    fn try_from(value: Vec<Vec<IndexMap<String, String>>>) -> XRPLCoreResult<Self, Self::Error> {
        if _is_path_set(&value) {
            // An empty path set, or a path with no steps, is not encodable: see
            // `Path::from_parser`.
            if value.is_empty() || value.iter().any(|path| path.is_empty()) {
                return Err(XRPLBinaryCodecException::EmptyPathInPathSet.into());
            }

            let mut buffer: Vec<u8> = vec![];

            for path_val in value {
                let result = Path::try_from(path_val);

                if let Ok(path) = result {
                    buffer.extend_from_slice(path.as_ref());
                    buffer.extend_from_slice(&[_PATH_SEPARATOR_BYTE; 1]);
                } else {
                    return Err(XRPLBinaryCodecException::InvalidPathSetFromValue.into());
                }
            }

            let len = buffer.len();
            buffer[len - 1] = _PATHSET_END_BYTE;

            PathSet::new(Some(&buffer))
        } else {
            Err(XRPLBinaryCodecException::InvalidPathSetFromValue.into())
        }
    }
}

impl TryFrom<&str> for Path {
    type Error = XRPLCoreException;

    /// Construct a Path object from a string.
    fn try_from(value: &str) -> XRPLCoreResult<Self, Self::Error> {
        let json: Vec<IndexMap<String, String>> =
            serde_json::from_str(value).map_err(XRPLSerdeJsonError::from)?;
        Self::try_from(json)
    }
}

impl TryFrom<&str> for PathStep {
    type Error = XRPLCoreException;

    /// Construct a PathSet object from a string.
    fn try_from(value: &str) -> XRPLCoreResult<Self, Self::Error> {
        let json: IndexMap<String, String> =
            serde_json::from_str(value).map_err(XRPLSerdeJsonError::from)?;
        Self::try_from(json)
    }
}

impl TryFrom<&str> for PathSet {
    type Error = XRPLCoreException;

    /// Construct a PathSet object from a string.
    fn try_from(value: &str) -> XRPLCoreResult<Self, Self::Error> {
        let json: Vec<Vec<IndexMap<String, String>>> =
            serde_json::from_str(value).map_err(XRPLSerdeJsonError::from)?;
        Self::try_from(json)
    }
}

impl AsRef<[u8]> for PathStep {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Path {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for PathSet {
    /// Get a reference of the byte representation.
    fn as_ref(&self) -> &[u8] {
        &self.0
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
            self.index = _TYPE_MPT;
            Some((_CUR_KEY.to_string(), self.currency.to_owned()))
        } else if self.index & _TYPE_MPT != 0 {
            self.index = _TYPE_ISSUER;
            Some((_MPT_KEY.to_string(), self.mpt_issuance_id.to_owned()))
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
    use crate::core::binarycodec::types::test_cases::TEST_PATH_BUFFER;
    use crate::core::binarycodec::types::test_cases::TEST_PATH_SET_BUFFER;
    use crate::core::binarycodec::types::test_cases::TEST_PATH_STEP_BUFFER;

    pub const PATH_SET_TEST: &str = include_str!("../test_data/path-set-test.json");
    pub const PATH_TEST: &str = include_str!("../test_data/path-test.json");

    #[test]
    fn test_is_path_step() {
        let json: Vec<Vec<IndexMap<String, String>>> =
            serde_json::from_str(PATH_SET_TEST).expect("");
        assert!(_is_path_set(&json));
    }

    #[test]
    fn test_is_path_set() {
        let json: Vec<Vec<IndexMap<String, String>>> =
            serde_json::from_str(PATH_SET_TEST).expect("");

        for path in json {
            for step in path {
                assert!(_is_path_step(&step));
            }
        }
    }

    #[test]
    fn test_pathstep_new() {
        for data in TEST_PATH_STEP_BUFFER {
            let hex: Vec<u8> = hex::decode(data).expect("");
            let pathstep = PathStep::new(Some(&hex)).unwrap();

            assert_eq!(pathstep.as_ref(), hex);
        }
    }

    #[test]
    fn test_path_new() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_BUFFER).expect("");
        let path = Path::new(Some(&hex)).unwrap();

        assert_eq!(hex::encode_upper(path.as_ref()), TEST_PATH_BUFFER);
    }

    #[test]
    fn test_pathset_new() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_SET_BUFFER).expect("");
        let pathset = PathSet::new(Some(&hex)).unwrap();

        assert_eq!(hex::encode_upper(pathset.as_ref()), TEST_PATH_SET_BUFFER);
    }

    #[test]
    fn test_pathstep_from_parser() {
        for data in TEST_PATH_STEP_BUFFER {
            let hex = hex::decode(data).expect("");
            let mut parser = BinaryParser::from(hex.clone());
            let pathset = PathStep::from_parser(&mut parser, None).unwrap();

            assert_eq!(pathset.as_ref(), hex);
        }
    }

    #[test]
    fn test_path_from_parser() {
        let hex = hex::decode(TEST_PATH_BUFFER).expect("");
        let mut parser = BinaryParser::from(hex.clone());
        let pathset = Path::from_parser(&mut parser, None).unwrap();

        assert_eq!(pathset.as_ref(), hex);
    }

    #[test]
    fn test_pathset_from_parser() {
        let hex = hex::decode(TEST_PATH_SET_BUFFER).expect("");
        let mut parser = BinaryParser::from(hex.clone());
        let pathset = PathSet::from_parser(&mut parser, None).unwrap();

        assert_eq!(pathset.as_ref(), hex);
    }

    #[test]
    fn test_pathstep_try_from() {
        let json: Vec<IndexMap<String, String>> = serde_json::from_str(PATH_TEST).expect("");
        let mut pathsteps: Vec<u8> = vec![];

        for map in json {
            pathsteps.extend_from_slice(PathStep::try_from(map.clone()).unwrap().as_ref());
        }

        assert_eq!(hex::encode_upper(pathsteps), TEST_PATH_BUFFER);
    }

    #[test]
    fn test_path_try_from() {
        let hex = hex::decode(TEST_PATH_BUFFER).expect("");
        let path = Path::try_from(PATH_TEST).unwrap();

        assert_eq!(path.as_ref(), hex)
    }

    #[test]
    fn test_pathset_try_from() {
        let hex = hex::decode(TEST_PATH_SET_BUFFER).expect("");
        let pathset = PathSet::try_from(PATH_SET_TEST).unwrap();

        assert_eq!(pathset.as_ref(), hex)
    }

    #[test]
    fn test_pathstep_to_json() {
        let json: Vec<IndexMap<String, String>> = serde_json::from_str(PATH_TEST).expect("");

        for map in json {
            let pathstep = PathStep::try_from(map.clone()).unwrap();
            assert_eq!(
                serde_json::to_string(&pathstep).unwrap(),
                serde_json::to_string(&map).expect(""),
            );
        }
    }

    #[test]
    fn test_path_to_json() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_BUFFER).unwrap();
        let path = Path::new(Some(&hex)).unwrap();
        let compact: serde_json::Value = serde_json::from_str(PATH_TEST).unwrap();

        assert_eq!(
            serde_json::to_string(&path).unwrap(),
            serde_json::to_string(&compact).expect(""),
        );
    }

    const MPT_ISSUANCE_ID: &str = "00002403C84A0A28E0190E208E982C352BBD5006600555CF";
    const TEST_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

    /// `0x40` ‖ 24-byte issuance id: the step `rippled` writes and this codec
    /// previously skipped without consuming the asset.
    #[test]
    fn test_pathstep_mpt_roundtrip() {
        let mut map: IndexMap<String, String> = IndexMap::new();
        map.insert(_MPT_KEY.to_string(), MPT_ISSUANCE_ID.to_string());

        let pathstep = PathStep::try_from(map.clone()).unwrap();
        let expected = alloc::format!("40{}", MPT_ISSUANCE_ID);

        assert_eq!(hex::encode_upper(pathstep.as_ref()), expected);
        // 1 type byte + 24 bytes of asset, not 1 byte.
        assert_eq!(pathstep.as_ref().len(), 1 + HASH192_LENGTH);
        assert_eq!(
            serde_json::to_string(&pathstep).unwrap(),
            serde_json::to_string(&map).unwrap(),
        );
    }

    #[test]
    fn test_pathstep_account_and_mpt_roundtrip() {
        let mut map: IndexMap<String, String> = IndexMap::new();
        map.insert(_ACC_KEY.to_string(), TEST_ACCOUNT.to_string());
        map.insert(_MPT_KEY.to_string(), MPT_ISSUANCE_ID.to_string());

        let pathstep = PathStep::try_from(map.clone()).unwrap();

        assert_eq!(pathstep.as_ref()[0], _TYPE_ACCOUNT | _TYPE_MPT);
        assert_eq!(
            pathstep.as_ref().len(),
            1 + ACCOUNT_ID_LENGTH + HASH192_LENGTH
        );
        assert_eq!(
            serde_json::to_string(&pathstep).unwrap(),
            serde_json::to_string(&map).unwrap(),
        );
    }

    #[test]
    fn test_pathstep_mpt_is_recognised_as_a_path_step() {
        let mut map: IndexMap<String, String> = IndexMap::new();
        map.insert(_MPT_KEY.to_string(), MPT_ISSUANCE_ID.to_string());

        assert!(_is_path_step(&map));
        assert!(_is_path_set(&[vec![map]]));
    }

    #[test]
    fn test_pathstep_from_parser_rejects_unknown_type_bits() {
        // Every bit outside TypeAll, on its own.
        for type_byte in [0x02u8, 0x04, 0x08, 0x80] {
            let mut parser = BinaryParser::from(vec![type_byte]);
            let err = PathStep::from_parser(&mut parser, None).unwrap_err();

            assert!(
                alloc::format!("{}", err).contains("Unknown path step type"),
                "type byte {:#04X} was not rejected: {}",
                type_byte,
                err,
            );
        }
    }

    #[test]
    fn test_pathstep_from_parser_rejects_mutually_exclusive_types() {
        // Currency|MPT, with and without Account. `rippled` throws on exactly this
        // pair and no other, so nothing else belongs in this list.
        for type_byte in [0x50u8, 0x51] {
            let mut buffer = vec![type_byte];
            buffer.extend_from_slice(&[0u8; 64]);

            let mut parser = BinaryParser::from(buffer);
            let err = PathStep::from_parser(&mut parser, None).unwrap_err();

            assert!(
                alloc::format!("{}", err).contains("Mutually exclusive"),
                "type byte {:#04X} was not rejected: {}",
                type_byte,
                err,
            );
        }
    }

    #[test]
    fn test_pathstep_try_from_rejects_currency_alongside_mpt() {
        let mut map: IndexMap<String, String> = IndexMap::new();
        map.insert(_MPT_KEY.to_string(), MPT_ISSUANCE_ID.to_string());
        map.insert(_CUR_KEY.to_string(), "USD".to_string());

        let err = PathStep::try_from(map).unwrap_err();

        assert!(
            alloc::format!("{}", err).contains("Mutually exclusive"),
            "currency alongside mpt_issuance_id was not rejected: {}",
            err,
        );
    }

    /// `rippled` accepts `Issuer` alongside `MPT` — it reads the issuance id and
    /// then the issuer — so such a step exists on-ledger and must decode here.
    /// Rejecting it would be the same codec/node disagreement in the other
    /// direction: a transaction the network validated that this codec cannot read.
    #[test]
    fn test_pathstep_accepts_issuer_alongside_mpt() {
        let mut map: IndexMap<String, String> = IndexMap::new();
        map.insert(_MPT_KEY.to_string(), MPT_ISSUANCE_ID.to_string());
        map.insert(_ISS_KEY.to_string(), TEST_ACCOUNT.to_string());

        let pathstep = PathStep::try_from(map.clone()).unwrap();

        assert_eq!(pathstep.as_ref()[0], _TYPE_ISSUER | _TYPE_MPT);
        assert_eq!(
            pathstep.as_ref().len(),
            1 + HASH192_LENGTH + ACCOUNT_ID_LENGTH
        );
        assert_eq!(
            serde_json::to_string(&pathstep).unwrap(),
            serde_json::to_string(&map).unwrap(),
        );

        // And the same bytes survive a parse.
        let mut parser = BinaryParser::from(pathstep.as_ref());
        let reparsed = PathStep::from_parser(&mut parser, None).unwrap();
        assert_eq!(reparsed.as_ref(), pathstep.as_ref());
    }

    /// A leading 0x00 or 0xFF terminates the path; it is never a zero-field step.
    #[test]
    fn test_path_from_parser_rejects_an_empty_path() {
        for terminator in [_PATHSET_END_BYTE, _PATH_SEPARATOR_BYTE] {
            let mut parser = BinaryParser::from(vec![terminator]);
            let err = Path::from_parser(&mut parser, None).unwrap_err();

            assert!(
                alloc::format!("{}", err).contains("Empty path"),
                "a bare {:#04X} was not rejected: {}",
                terminator,
                err,
            );
        }
    }

    /// The encoder must refuse what the decoder refuses, or it emits blobs this
    /// crate cannot read back.
    #[test]
    fn test_pathset_try_from_rejects_empty_paths() {
        for json in [
            "[]",
            "[[]]",
            r#"[[{"account":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"}],[]]"#,
        ] {
            let err = PathSet::try_from(json).unwrap_err();

            assert!(
                alloc::format!("{}", err).contains("Empty path"),
                "{} was not rejected: {}",
                json,
                err,
            );
        }
    }

    #[test]
    fn test_pathset_to_json() {
        let hex: Vec<u8> = hex::decode(TEST_PATH_SET_BUFFER).expect("");
        let compact: serde_json::Value = serde_json::from_str(PATH_SET_TEST).expect("");
        let pathset = PathSet::new(Some(&hex)).unwrap();

        assert_eq!(
            serde_json::to_string(&pathset).unwrap(),
            serde_json::to_string(&compact).expect(""),
        );
    }
}
