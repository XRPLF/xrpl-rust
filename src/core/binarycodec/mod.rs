//! Functions for encoding objects into the XRP Ledger's
//! canonical binary format and decoding them.
//!
//! This module is the public API entry point.
//! Internal serialization/deserialization logic lives in `binary_wrappers`

pub mod definitions;
pub mod types;

use types::AccountId;

use alloc::borrow::Cow;
use alloc::string::String;
use core::convert::TryFrom;
use serde::Serialize;
use serde_json::Value;

pub mod binary_wrappers;
pub mod exceptions;
pub(crate) mod test_cases;
pub mod utils;

pub use binary_wrappers::*;

use self::binary_wrappers::{
    decode_ledger_data_inner, decode_st_object, serialize_json, TRANSACTION_MULTISIG_PREFIX,
    TRANSACTION_SIGNATURE_PREFIX,
};

use super::exceptions::XRPLCoreResult;

/// Encode a transaction (or any XRPL object) to hex-encoded binary.
pub fn encode<T>(signed_transaction: &T) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    serialize_json(signed_transaction, None, None, false)
}

/// Encode a transaction for signing (prepends the signing prefix).
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

/// Encode a transaction for multi-signing (prepends multi-sign prefix,
/// appends the signing account ID).
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

/// Decode a hex-encoded XRPL binary blob into a JSON object.
///
/// This is the inverse of `encode`. It takes a hex string representing
/// a serialized XRPL transaction (or other object) and returns its
/// JSON representation as a `serde_json::Value`.
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
    decode_ledger_data_inner(hex_string)
}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;

    #[path = "binary_json_tests.rs"]
    mod binary_json_tests;
    #[path = "binary_serializer_tests.rs"]
    mod binary_serializer_tests;
    #[path = "tx_encode_decode_tests.rs"]
    mod tx_encode_decode_tests;
    #[path = "x_address_tests.rs"]
    mod x_address_tests;
}
