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
    decode_ledger_data_inner, decode_st_object, serialize_json, BATCH_PREFIX,
    PAYMENT_CHANNEL_CLAIM_PREFIX, TRANSACTION_MULTISIG_PREFIX, TRANSACTION_SIGNATURE_PREFIX,
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

/// Encode a payment channel claim for signing.
///
/// This produces the serialized data that must be signed to authorize
/// a claim against a payment channel. The format is:
/// - 4 bytes: HashPrefix `0x434C4D00` ("CLM\0")
/// - 32 bytes: channel ID (Hash256)
/// - 8 bytes: amount in drops (UInt64, big-endian)
///
/// See Payment Channel Claim:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelclaim>`
pub fn encode_for_signing_claim(channel: &str, amount: &str) -> XRPLCoreResult<String> {
    let channel_bytes = hex::decode(channel).map_err(|_| {
        super::exceptions::XRPLCoreException::XRPLBinaryCodecError(
            exceptions::XRPLBinaryCodecException::InvalidHashLength {
                expected: 64,
                found: channel.len(),
            },
        )
    })?;
    if channel_bytes.len() != 32 {
        return Err(super::exceptions::XRPLCoreException::XRPLBinaryCodecError(
            exceptions::XRPLBinaryCodecException::InvalidHashLength {
                expected: 32,
                found: channel_bytes.len(),
            },
        ));
    }
    let amount_val: u64 = amount.parse().map_err(|e| {
        super::exceptions::XRPLCoreException::XRPLBinaryCodecError(
            exceptions::XRPLBinaryCodecException::ParseIntError(e),
        )
    })?;

    let mut buf = alloc::vec::Vec::with_capacity(44);
    buf.extend_from_slice(&PAYMENT_CHANNEL_CLAIM_PREFIX);
    buf.extend_from_slice(&channel_bytes);
    buf.extend_from_slice(&amount_val.to_be_bytes());
    Ok(hex::encode_upper(&buf))
}

/// Encode a Batch transaction for signing.
///
/// This produces the serialized data that must be signed to authorize
/// a batch transaction. The format is:
/// - 4 bytes: HashPrefix `0x42434800` ("BCH\0")
/// - 4 bytes: flags (UInt32, big-endian)
/// - 4 bytes: number of txIDs (UInt32, big-endian)
/// - N × 32 bytes: each txID (Hash256)
///
/// See Batch Transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/batch>`
pub fn encode_for_signing_batch(flags: u32, tx_ids: &[&str]) -> XRPLCoreResult<String> {
    let mut buf = alloc::vec::Vec::with_capacity(4 + 4 + 4 + tx_ids.len() * 32);
    buf.extend_from_slice(&BATCH_PREFIX);
    buf.extend_from_slice(&flags.to_be_bytes());
    buf.extend_from_slice(&(tx_ids.len() as u32).to_be_bytes());
    for tx_id in tx_ids {
        let id_bytes = hex::decode(tx_id).map_err(|_| {
            super::exceptions::XRPLCoreException::XRPLBinaryCodecError(
                exceptions::XRPLBinaryCodecException::InvalidHashLength {
                    expected: 64,
                    found: tx_id.len(),
                },
            )
        })?;
        if id_bytes.len() != 32 {
            return Err(super::exceptions::XRPLCoreException::XRPLBinaryCodecError(
                exceptions::XRPLBinaryCodecException::InvalidHashLength {
                    expected: 32,
                    found: id_bytes.len(),
                },
            ));
        }
        buf.extend_from_slice(&id_bytes);
    }
    Ok(hex::encode_upper(&buf))
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
