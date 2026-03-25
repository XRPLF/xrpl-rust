//! Functions for encoding objects into the XRP Ledger's
//! canonical binary format and decoding them.

pub mod definitions;
pub mod types;

use types::{
    AccountId, Amount, Blob, Hash128, Hash160, Hash256, Issue, PathSet, STObject, TryFromParser,
    Vector256, XChainBridge,
};

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::convert::TryFrom;
use hex::ToHex;
use serde::Serialize;
use serde_json::{Map, Value};

pub mod binary_wrappers;
pub mod exceptions;
pub(crate) mod test_cases;
pub mod utils;

pub use binary_wrappers::*;

use crate::XRPLSerdeJsonError;

use self::definitions::{
    get_delegatable_permission_name, get_ledger_entry_type_name, get_transaction_result_name,
    get_transaction_type_name, FieldInstance,
};

use super::exceptions::XRPLCoreResult;

const TRANSACTION_SIGNATURE_PREFIX: i32 = 0x53545800;
const TRANSACTION_MULTISIG_PREFIX: [u8; 4] = (0x534D5400u32).to_be_bytes();

pub fn encode<T>(signed_transaction: &T) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    serialize_json(signed_transaction, None, None, false)
}

pub fn encode_for_signing<T>(prepared_transaction: &T) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_SIGNATURE_PREFIX.to_be_bytes().as_ref()),
        None,
        true,
    )
}

pub fn encode_for_multisigning<T>(
    prepared_transaction: &T,
    signing_account: Cow<'_, str>,
) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    let signing_account_id = AccountId::try_from(signing_account.as_ref()).unwrap();

    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_MULTISIG_PREFIX.as_ref()),
        Some(signing_account_id.as_ref()),
        true,
    )
}

fn serialize_json<T>(
    prepared_transaction: &T,
    prefix: Option<&[u8]>,
    suffix: Option<&[u8]>,
    signing_only: bool,
) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    if let Some(p) = prefix {
        buffer.extend(p);
    }

    let json_value =
        serde_json::to_value(prepared_transaction).map_err(XRPLSerdeJsonError::from)?;
    // dbg!(&json_value);
    let st_object = STObject::try_from_value(json_value, signing_only)?;
    buffer.extend(st_object.as_ref());

    if let Some(s) = suffix {
        buffer.extend(s);
    }
    let hex_string = buffer.encode_hex_upper::<String>();

    Ok(hex_string)
}

/// UInt64 fields that should be decoded as base-10 strings instead of hex.
const BASE10_UINT64_FIELDS: &[&str] = &[
    "MaximumAmount",
    "OutstandingAmount",
    "MPTAmount",
    "LockedAmount",
];

/// Decode a single field value from a BinaryParser based on the field's type.
/// Returns the JSON value for the field.
fn decode_field_value(parser: &mut BinaryParser, field: &FieldInstance) -> XRPLCoreResult<Value> {
    let type_name = field.associated_type.as_str();

    // Handle VL prefix for variable-length encoded fields
    let length = if field.is_vl_encoded {
        Some(parser.read_length_prefix()?)
    } else {
        None
    };

    match type_name {
        "AccountID" => {
            let account = AccountId::from_parser(parser, length)?;
            Ok(serde_json::to_value(&account).map_err(XRPLSerdeJsonError::from)?)
        }
        "Amount" => {
            let amount = Amount::from_parser(parser, length)?;
            Ok(serde_json::to_value(&amount).map_err(XRPLSerdeJsonError::from)?)
        }
        "Blob" => {
            let blob = Blob::from_parser(parser, length)?;
            Ok(serde_json::to_value(&blob).map_err(XRPLSerdeJsonError::from)?)
        }
        "Hash128" => {
            let hash = Hash128::from_parser(parser, length)?;
            Ok(serde_json::to_value(&hash).map_err(XRPLSerdeJsonError::from)?)
        }
        "Hash160" => {
            let hash = Hash160::from_parser(parser, length)?;
            Ok(serde_json::to_value(&hash).map_err(XRPLSerdeJsonError::from)?)
        }
        "Hash256" => {
            let hash = Hash256::from_parser(parser, length)?;
            Ok(serde_json::to_value(&hash).map_err(XRPLSerdeJsonError::from)?)
        }
        "UInt8" => {
            let val = parser.read_uint8()?;
            // TransactionResult is stored as UInt8 in some contexts
            if field.name == "TransactionResult" {
                let code = val as i16;
                if let Some(name) = get_transaction_result_name(&code) {
                    return Ok(Value::String(name.clone()));
                }
            }
            Ok(Value::Number(val.into()))
        }
        "UInt16" => {
            let val = parser.read_uint16()?;
            // Special fields: decode numeric codes to string names
            if field.name == "TransactionType" {
                let code = val as i16;
                if let Some(name) = get_transaction_type_name(&code) {
                    return Ok(Value::String(name.clone()));
                }
            } else if field.name == "LedgerEntryType" {
                let code = val as i16;
                if let Some(name) = get_ledger_entry_type_name(&code) {
                    return Ok(Value::String(name.clone()));
                }
            } else if field.name == "TransactionResult" {
                let code = val as i16;
                if let Some(name) = get_transaction_result_name(&code) {
                    return Ok(Value::String(name.clone()));
                }
            }
            Ok(Value::Number(val.into()))
        }
        "UInt32" => {
            let val = parser.read_uint32()?;
            if field.name == "PermissionValue" {
                let code = val as i32;
                if let Some(name) = get_delegatable_permission_name(&code) {
                    return Ok(Value::String(name.clone()));
                }
            }
            Ok(Value::Number(val.into()))
        }
        "UInt64" => {
            let bytes = parser.read(8)?;
            if BASE10_UINT64_FIELDS.contains(&field.name.as_str()) {
                // Decode as base-10 string
                let val = u64::from_be_bytes(bytes.as_slice().try_into().map_err(|_| {
                    exceptions::XRPLBinaryCodecException::InvalidReadFromBytesValue
                })?);
                Ok(Value::String(val.to_string()))
            } else {
                // Decode as uppercase hex string
                Ok(Value::String(hex::encode_upper(&bytes)))
            }
        }
        "STObject" => decode_st_object(parser, true),
        "STArray" => decode_st_array(parser),
        "PathSet" => {
            let path_set = PathSet::from_parser(parser, length)?;
            Ok(serde_json::to_value(&path_set).map_err(XRPLSerdeJsonError::from)?)
        }
        "Vector256" => {
            let vector = Vector256::from_parser(parser, length)?;
            Ok(serde_json::to_value(&vector).map_err(XRPLSerdeJsonError::from)?)
        }
        "Currency" => {
            let currency = crate::core::binarycodec::types::Currency::from_parser(parser, length)?;
            Ok(serde_json::to_value(&currency).map_err(XRPLSerdeJsonError::from)?)
        }
        "Issue" => {
            let issue = Issue::from_parser(parser, length)?;
            Ok(serde_json::to_value(&issue).map_err(XRPLSerdeJsonError::from)?)
        }
        "XChainBridge" => {
            let bridge = XChainBridge::from_parser(parser, length)?;
            Ok(serde_json::to_value(&bridge).map_err(XRPLSerdeJsonError::from)?)
        }
        "Number" => {
            let number = crate::core::binarycodec::types::Number::from_parser(parser, length)?;
            Ok(serde_json::to_value(&number).map_err(XRPLSerdeJsonError::from)?)
        }
        _ => {
            // Unknown type: read remaining bytes as hex blob if length is known
            if let Some(len) = length {
                let bytes = parser.read(len)?;
                Ok(Value::String(hex::encode_upper(&bytes)))
            } else {
                Ok(Value::Null)
            }
        }
    }
}

/// Decode an STObject from the parser. Reads fields until ObjectEndMarker (0xE1)
/// or end of parser data.
///
/// `is_inner` indicates whether this is an inner object (will stop at ObjectEndMarker)
/// or a top-level object (will read until parser is empty).
fn decode_st_object(parser: &mut BinaryParser, _is_inner: bool) -> XRPLCoreResult<Value> {
    let mut accumulator = Map::new();

    while !parser.is_end(None) {
        let field = parser.read_field()?;

        if field.name == "ObjectEndMarker" {
            break;
        }

        let value = decode_field_value(parser, &field)?;
        accumulator.insert(field.name, value);
    }

    Ok(Value::Object(accumulator))
}

/// Decode an STArray from the parser. Reads wrapper objects until
/// ArrayEndMarker (0xF1) or end of parser data.
fn decode_st_array(parser: &mut BinaryParser) -> XRPLCoreResult<Value> {
    let mut result: Vec<Value> = Vec::new();

    while !parser.is_end(None) {
        let field = parser.read_field()?;

        if field.name == "ArrayEndMarker" {
            break;
        }

        // Each array element is wrapped: { "FieldName": { inner object } }
        let inner = decode_st_object(parser, true)?;
        let mut wrapper = Map::new();
        wrapper.insert(field.name, inner);
        result.push(Value::Object(wrapper));
    }

    Ok(Value::Array(result))
}

/// Decode a hex-encoded XRPL binary blob into a JSON object.
///
/// This is the inverse of `encode`. It takes a hex string representing
/// a serialized XRPL transaction (or other object) and returns its
/// JSON representation as a `serde_json::Value`.
///
/// # Examples
///
/// ```
/// use xrpl::core::binarycodec::{encode, decode};
/// use serde_json::json;
///
/// let tx = json!({
///     "Account": "r3kmLJN5D28dHuH8vZNUZpMC43pEHpaocV",
///     "Destination": "rLQBHVhFnaC5gLEkgr6HgBJJ3bgeZHg9cj",
///     "TransactionType": "Payment",
///     "Amount": "10000000000",
///     "Fee": "10",
///     "Flags": 0,
///     "Sequence": 62,
///     "SigningPubKey": "034AADB09CFF4A4804073701EC53C3510CDC95917C2BB0150FB742D0C66E6CEE9E",
///     "TxnSignature": "3045022022EB32AECEF7C644C891C19F87966DF9C62B1F34BABA6BE774325E4BB8E2DD62022100A51437898C28C2B297112DF8131F2BB39EA5FE613487DDD611525F17962646398114550FC62003E785DC231A1058A05E56E3F09CF4E68314D4CC8AB5B21D86A82C3E9E8D0ECF2404B77FECBA"
/// });
/// ```
pub fn decode(hex_string: &str) -> XRPLCoreResult<Value> {
    let mut parser = BinaryParser::try_from(hex_string)?;
    decode_st_object(&mut parser, false)
}

/// Decode a serialized ledger header from hex into JSON.
///
/// Ledger headers use a fixed-length format (not field-prefixed like STObject):
/// - 4 bytes: ledger_index (UInt32)
/// - 8 bytes: total_coins (UInt64, as base-10 string)
/// - 32 bytes: parent_hash (Hash256)
/// - 32 bytes: transaction_hash (Hash256)
/// - 32 bytes: account_hash (Hash256)
/// - 4 bytes: parent_close_time (UInt32)
/// - 4 bytes: close_time (UInt32)
/// - 1 byte: close_time_resolution (UInt8)
/// - 1 byte: close_flags (UInt8)
pub fn decode_ledger_data(hex_string: &str) -> XRPLCoreResult<Value> {
    let mut parser = BinaryParser::try_from(hex_string)?;

    let ledger_index = parser.read_uint32()?;

    let coins_bytes = parser.read(8)?;
    let total_coins = u64::from_be_bytes(
        coins_bytes
            .as_slice()
            .try_into()
            .map_err(|_| exceptions::XRPLBinaryCodecException::InvalidReadFromBytesValue)?,
    );

    let parent_hash_bytes = parser.read(32)?;
    let transaction_hash_bytes = parser.read(32)?;
    let account_hash_bytes = parser.read(32)?;

    let parent_close_time = parser.read_uint32()?;
    let close_time = parser.read_uint32()?;
    let close_time_resolution = parser.read_uint8()?;
    let close_flags = parser.read_uint8()?;

    let mut map = Map::new();
    map.insert("ledger_index".into(), Value::Number(ledger_index.into()));
    map.insert("total_coins".into(), Value::String(total_coins.to_string()));
    map.insert(
        "parent_hash".into(),
        Value::String(hex::encode_upper(&parent_hash_bytes)),
    );
    map.insert(
        "transaction_hash".into(),
        Value::String(hex::encode_upper(&transaction_hash_bytes)),
    );
    map.insert(
        "account_hash".into(),
        Value::String(hex::encode_upper(&account_hash_bytes)),
    );
    map.insert(
        "parent_close_time".into(),
        Value::Number(parent_close_time.into()),
    );
    map.insert("close_time".into(), Value::Number(close_time.into()));
    map.insert(
        "close_time_resolution".into(),
        Value::Number(close_time_resolution.into()),
    );
    map.insert("close_flags".into(), Value::Number(close_flags.into()));

    Ok(Value::Object(map))
}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;

    mod binary_json_tests;
    mod binary_serializer_tests;
    mod tx_encode_decode_tests;
    mod x_address_tests;
}
