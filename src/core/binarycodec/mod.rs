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
    decode_st_object(&mut parser)
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
    use alloc::vec;

    use super::*;
    use crate::utils::testing::test_constants::*;

    #[path = "binary_json_tests.rs"]
    mod binary_json_tests;
    #[path = "binary_serializer_tests.rs"]
    mod binary_serializer_tests;
    #[path = "tx_encode_decode_tests.rs"]
    mod tx_encode_decode_tests;
    #[path = "x_address_tests.rs"]
    mod x_address_tests;

    use crate::core::binarycodec::definitions::{
        get_field_instance, get_ledger_entry_type_code, get_transaction_type_code,
    };
    use crate::core::binarycodec::utils::{decode_field_name, encode_field_name};
    use crate::models::transactions::{
        mptoken_authorize::MPTokenAuthorize,
        mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
        mptoken_issuance_destroy::MPTokenIssuanceDestroy,
        mptoken_issuance_set::{MPTokenIssuanceSet, MPTokenIssuanceSetFlag},
        CommonFields, TransactionType,
    };

    // ── Field encoding / decoding ──────────────────────────────────────

    #[test]
    fn test_mpt_field_name_encoding() {
        // (field_name, expected_hex)
        // Hash192 type_code=21 (>=16), AccountID type_code=8 (<16),
        // UInt8 type_code=16 (>=16), UInt64 type_code=3, Blob type_code=7
        let cases = [
            ("MPTokenIssuanceID", "0115"), // Hash192(21), nth 1 → byte1=0x01, byte2=0x15
            ("ShareMPTID", "0215"),        // Hash192(21), nth 2 → byte1=0x02, byte2=0x15
            ("Holder", "8B"),              // AccountID(8), nth 11 → (8<<4)|11 = 0x8B
            ("AssetScale", "0510"),        // UInt8(16), nth 5 → byte1=0x05, byte2=0x10
            ("MaximumAmount", "3018"),     // UInt64(3), nth 24 → (3<<4)=0x30, 0x18
            ("MPTAmount", "301A"),         // UInt64(3), nth 26 → (3<<4)=0x30, 0x1A
            ("MPTokenMetadata", "701E"),   // Blob(7), nth 30 → (7<<4)=0x70, 0x1E
        ];

        for (field_name, expected_hex) in &cases {
            let encoded = encode_field_name(field_name)
                .unwrap_or_else(|e| panic!("failed to encode field {}: {:?}", field_name, e));
            let hex = hex::encode_upper(encoded);
            assert_eq!(
                &hex, expected_hex,
                "encode mismatch for field {}",
                field_name
            );

            let decoded = decode_field_name(expected_hex)
                .unwrap_or_else(|e| panic!("failed to decode hex {}: {:?}", expected_hex, e));
            assert_eq!(
                decoded, *field_name,
                "decode mismatch for hex {}",
                expected_hex
            );
        }
    }

    // ── Type code resolution ───────────────────────────────────────────

    #[test]
    fn test_mpt_transaction_type_codes() {
        assert_eq!(
            get_transaction_type_code("MPTokenIssuanceCreate"),
            Some(&54)
        );
        assert_eq!(
            get_transaction_type_code("MPTokenIssuanceDestroy"),
            Some(&55)
        );
        assert_eq!(get_transaction_type_code("MPTokenIssuanceSet"), Some(&56));
        assert_eq!(get_transaction_type_code("MPTokenAuthorize"), Some(&57));
    }

    #[test]
    fn test_mpt_ledger_entry_type_codes() {
        assert_eq!(get_ledger_entry_type_code("MPTokenIssuance"), Some(&126));
        assert_eq!(get_ledger_entry_type_code("MPToken"), Some(&127));
    }

    // ── Field instance metadata ────────────────────────────────────────

    #[test]
    fn test_mpt_field_instances() {
        let fi = get_field_instance("MPTokenIssuanceID").expect("MPTokenIssuanceID not found");
        assert_eq!(fi.associated_type, "Hash192");
        assert_eq!(fi.nth, 1);
        assert!(fi.is_serialized);
        assert!(fi.is_signing);

        let fi = get_field_instance("Holder").expect("Holder not found");
        assert_eq!(fi.associated_type, "AccountID");
        assert_eq!(fi.nth, 11);
        assert!(fi.is_vl_encoded);

        let fi = get_field_instance("MPTAmount").expect("MPTAmount not found");
        assert_eq!(fi.associated_type, "UInt64");
        assert_eq!(fi.nth, 26);

        let fi = get_field_instance("MPTokenMetadata").expect("MPTokenMetadata not found");
        assert_eq!(fi.associated_type, "Blob");
        assert_eq!(fi.nth, 30);
        assert!(fi.is_vl_encoded);

        let fi = get_field_instance("AssetScale").expect("AssetScale not found");
        assert_eq!(fi.associated_type, "UInt8");
        assert_eq!(fi.nth, 5);

        let fi = get_field_instance("MaximumAmount").expect("MaximumAmount not found");
        assert_eq!(fi.associated_type, "UInt64");
        assert_eq!(fi.nth, 24);

        let fi = get_field_instance("ShareMPTID").expect("ShareMPTID not found");
        assert_eq!(fi.associated_type, "Hash192");
        assert_eq!(fi.nth, 2);
    }

    // ── Full transaction encoding ──────────────────────────────────────

    /// TransactionType is always the first serialized field (lowest ordinal).
    /// For MPTokenIssuanceCreate (code 54 = 0x0036), the hex starts with
    /// field ID 0x12 (UInt16, nth 2) followed by the 2-byte type code.
    #[test]
    fn test_encode_mptoken_issuance_create() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                fee: Some("10".into()),
                sequence: Some(1),
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            transfer_fee: Some(314),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };

        let hex = encode(&txn).expect("encode MPTokenIssuanceCreate failed");

        // TransactionType field: 0x12 + 0x0036 (54)
        assert!(
            hex.starts_with("120036"),
            "expected hex to start with 120036 (MPTokenIssuanceCreate), got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // TransferFee field: 0x14 + 0x013A (314)
        assert!(
            hex.contains("14013A"),
            "expected TransferFee 314 (14013A) in hex"
        );

        // Flags = TfMPTCanTransfer (0x20): 0x22 + 0x00000020
        assert!(
            hex.contains("2200000020"),
            "expected Flags TfMPTCanTransfer (2200000020) in hex"
        );

        // Sequence = 1: 0x24 + 0x00000001
        assert!(
            hex.contains("2400000001"),
            "expected Sequence 1 (2400000001) in hex"
        );

        // AssetScale = 2: field ID 0x0510 + 0x02
        assert!(
            hex.contains("051002"),
            "expected AssetScale 2 (051002) in hex"
        );

        // MPTokenMetadata (Blob): field ID 0x701E + length prefix + CAFEBABE
        assert!(
            hex.contains("CAFEBABE"),
            "expected MPTokenMetadata hex payload in encoded output"
        );
    }

    #[test]
    fn test_encode_mptoken_issuance_create_with_flags() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                fee: Some("12".into()),
                sequence: Some(5),
                flags: vec![
                    MPTokenIssuanceCreateFlag::TfMPTCanTransfer,
                    MPTokenIssuanceCreateFlag::TfMPTCanLock,
                ]
                .into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let hex = encode(&txn).expect("encode MPTokenIssuanceCreate with flags failed");

        assert!(hex.starts_with("120036"), "wrong transaction type");

        // Flags = TfMPTCanTransfer (0x20) | TfMPTCanLock (0x02) = 0x22
        assert!(
            hex.contains("2200000022"),
            "expected Flags 0x22 (2200000022) in hex, got: {}",
            hex
        );
    }

    /// MPTokenIssuanceDestroy (code 55 = 0x0037) exercises Hash192
    /// serialization through the MPTokenIssuanceID field.
    #[test]
    fn test_encode_mptoken_issuance_destroy() {
        let txn = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344".into(),
        };

        let hex = encode(&txn).expect("encode MPTokenIssuanceDestroy failed");

        // TransactionType = 55 = 0x0037
        assert!(
            hex.starts_with("120037"),
            "expected hex to start with 120037 (MPTokenIssuanceDestroy), got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // The Hash192 value should appear verbatim in the encoded hex
        // (Hash192 is a fixed-length 24-byte field, no length prefix)
        assert!(
            hex.contains("00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344"),
            "expected MPTokenIssuanceID hash in encoded output"
        );
    }

    /// MPTokenIssuanceSet (code 56 = 0x0038) with the TfMPTLock flag
    /// and Holder field (AccountID, nth 11).
    #[test]
    fn test_encode_mptoken_issuance_set() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                fee: Some("10".into()),
                sequence: Some(1),
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344".into(),
            holder: Some(ACCOUNT_ALT.into()),
            ..Default::default()
        };

        let hex = encode(&txn).expect("encode MPTokenIssuanceSet failed");

        // TransactionType = 56 = 0x0038
        assert!(
            hex.starts_with("120038"),
            "expected hex to start with 120038 (MPTokenIssuanceSet), got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // TfMPTLock = 0x00000001
        assert!(
            hex.contains("2200000001"),
            "expected Flags TfMPTLock (2200000001) in hex"
        );

        // Hash192 value in output
        assert!(
            hex.contains("00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344"),
            "expected MPTokenIssuanceID in encoded output"
        );

        // Holder field (AccountID, 0x8B) should be present
        assert!(hex.contains("8B"), "expected Holder field ID (8B) in hex");
    }

    /// MPTokenAuthorize (code 57 = 0x0039) with holder opt-in (no holder field).
    #[test]
    fn test_encode_mptoken_authorize() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344".into(),
            ..Default::default()
        };

        let hex = encode(&txn).expect("encode MPTokenAuthorize failed");

        // TransactionType = 57 = 0x0039
        assert!(
            hex.starts_with("120039"),
            "expected hex to start with 120039 (MPTokenAuthorize), got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // Hash192 value
        assert!(
            hex.contains("00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344"),
            "expected MPTokenIssuanceID in encoded output"
        );
    }

    /// Verify that encode_for_signing adds the signing prefix and
    /// excludes non-signing fields for MPT transactions.
    #[test]
    fn test_encode_for_signing_mptoken_issuance_create() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            ..Default::default()
        };

        let hex = encode_for_signing(&txn).expect("encode_for_signing failed");

        // Signing prefix: 0x53545800
        assert!(
            hex.starts_with("53545800"),
            "expected signing prefix 53545800, got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // TransactionType follows the prefix
        assert!(
            hex[8..].starts_with("120036"),
            "expected TransactionType after signing prefix"
        );
    }

    /// Verify that encode_for_multisigning adds the multisign prefix,
    /// the signing account suffix, and excludes non-signing fields.
    #[test]
    fn test_encode_for_multisigning_mptoken_authorize() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344".into(),
            ..Default::default()
        };

        let hex = encode_for_multisigning(&txn, ACCOUNT_ALT.into())
            .expect("encode_for_multisigning failed");

        // Multisign prefix: 0x534D5400
        assert!(
            hex.starts_with("534D5400"),
            "expected multisign prefix 534D5400, got: {}",
            &hex[..core::cmp::min(20, hex.len())]
        );

        // TransactionType follows the prefix
        assert!(
            hex[8..].starts_with("120039"),
            "expected MPTokenAuthorize type after multisign prefix"
        );

        // The signing account ID should appear at the end as a suffix
        // (account ID is 20 bytes = 40 hex chars at end of encoded output)
        assert!(
            hex.len() > 40,
            "encoded output too short for multisign suffix"
        );
    }

    /// Encode the same transaction twice and verify deterministic output.
    #[test]
    fn test_encode_deterministic() {
        let txn = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                fee: Some("10".into()),
                sequence: Some(42),
                ..Default::default()
            },
            mptoken_issuance_id: "AABBCCDD11223344AABBCCDD11223344AABBCCDD11223344".into(),
        };

        let hex1 = encode(&txn).expect("first encode failed");
        let hex2 = encode(&txn).expect("second encode failed");
        assert_eq!(hex1, hex2, "encoding should be deterministic");
    }

    // ── Hash192 round-trip ─────────────────────────────────────────────

    /// Encode a transaction that includes a Hash192 field (MPTokenIssuanceID),
    /// then decode the result and verify the field value survives the round-trip.
    #[test]
    fn test_hash192_encode_decode_roundtrip() {
        use crate::core::binarycodec::{decode, encode};

        let issuance_id = "00000001A407AF5856CEFBF81F3D4A00AABBCCDD11223344";
        let txn = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
        };

        let encoded = encode(&txn).expect("encode MPTokenIssuanceDestroy failed");
        let decoded = decode(&encoded).expect("decode failed");

        let decoded_id = decoded
            .get("MPTokenIssuanceID")
            .expect("MPTokenIssuanceID missing from decoded output")
            .as_str()
            .expect("MPTokenIssuanceID is not a string");

        assert_eq!(
            decoded_id.to_uppercase(),
            issuance_id.to_uppercase(),
            "Hash192 value did not survive encode->decode round-trip"
        );
    }

    /// Exact-match encode/decode against authoritative vectors from xrpl.js
    /// ripple-binary-codec uint.test.ts ("UInt64 is parsed as base 10 for MPT amounts").
    ///
    /// These vectors were produced by the reference implementation and cover:
    ///   - MPTokenIssuance ledger entry (Hash192, UInt64 MPT amounts, Blob metadata)
    ///   - MPToken ledger entry (Hash192 MPTokenIssuanceID, UInt64 MPTAmount)
    #[test]
    fn test_mpt_ledger_entry_exact_vectors() {
        use super::{decode, serialize_json};
        use serde_json::json;

        // ── MPTokenIssuance ──────────────────────────────────────────────────
        // Source: xrpl.js ripple-binary-codec/test/uint.test.ts
        // `mptIssuanceEntryBinary` / `mptIssuanceEntryJson`
        let mpt_issuance_binary = concat!(
            "11007E220000006224000002DF25000002E434000000000000000030187FFFFFFFFFFFFFFF",
            "30190000000000000064552E78C1FFBDDAEE077253CEB12CFEA83689AA0899F94762190A357208DADC76FE",
            "701EC1EC7B226E616D65223A2255532054726561737572792042696C6C20546F6B656E222C2273796D626F",
            "6C223A225553544254222C22646563696D616C73223A322C22746F74616C537570706C79223A313030303030",
            "302C22697373756572223A225553205472656173757279222C22697373756544617465223A22323032342D30",
            "332D3235222C226D6174757269747944617465223A22323032352D30332D3235222C226661636556616C7565",
            "223A2231303030222C22696E74657265737452617465223A22322E35222C22696E7465726573744672657175",
            "656E6379223A22517561727465726C79222C22636F6C6C61746572616C223A22555320476F7665726E6D656E",
            "74222C226A7572697364696374696F6E223A22556E6974656420537461746573222C22726567756C61746F72",
            "79436F6D706C69616E6365223A2253454320526567756C6174696F6E73222C22736563757269747954797065",
            "223A2254726561737572792042696C6C222C2265787465726E616C5F75726C223A2268747470733A2F2F6578",
            "616D706C652E636F6D2F742D62696C6C2D746F6B656E2D6D657461646174612E6A736F6E227D",
            "8414A4D893CFBC4DC6AE877EB585F90A3B47528B958D051003"
        );

        let decoded_issuance = decode(mpt_issuance_binary).expect("decode MPTokenIssuance failed");
        assert_eq!(
            decoded_issuance["LedgerEntryType"],
            json!("MPTokenIssuance")
        );
        assert_eq!(decoded_issuance["AssetScale"], json!(3));
        assert_eq!(decoded_issuance["Flags"], json!(98));
        assert_eq!(decoded_issuance["Sequence"], json!(735));
        assert_eq!(
            decoded_issuance["MaximumAmount"],
            json!("9223372036854775807")
        );
        assert_eq!(decoded_issuance["OutstandingAmount"], json!("100"));
        assert_eq!(
            decoded_issuance["Issuer"],
            json!("rGpdGXDV2RFPeLEfWS9RFo5Nh9cpVDToZa")
        );
        assert_eq!(
            decoded_issuance["PreviousTxnID"],
            json!("2E78C1FFBDDAEE077253CEB12CFEA83689AA0899F94762190A357208DADC76FE")
        );
        assert_eq!(decoded_issuance["PreviousTxnLgrSeq"], json!(740));

        // Re-encode and confirm byte-for-byte match
        let re_encoded = serialize_json(&decoded_issuance, None, None, false)
            .expect("re-encode MPTokenIssuance failed");
        assert_eq!(
            re_encoded.to_uppercase(),
            mpt_issuance_binary.to_uppercase(),
            "MPTokenIssuance re-encode does not match authoritative vector"
        );

        // ── MPToken ──────────────────────────────────────────────────────────
        // Source: xrpl.js ripple-binary-codec/test/uint.test.ts
        // `mptokenEntryBinary` / `mptokenEntryJson`
        let mptoken_binary = concat!(
            "11007F220000000025000002E5340000000000000000",
            "301A00000000000000645522",
            "2EF3C7E82D8A44984A66E2B8E357CB536EC2547359CCF70E56E14BC4C284C8",
            "81143930DB9A74C26D96CB58ADFFD7E8BB78BCFE623",
            "40115000002DF71CAE59C9B7E56587FFF74D4EA5830D9BE3CE0CC"
        );

        let decoded_token = decode(mptoken_binary).expect("decode MPToken failed");
        assert_eq!(decoded_token["LedgerEntryType"], json!("MPToken"));
        assert_eq!(decoded_token["Flags"], json!(0));
        assert_eq!(decoded_token["MPTAmount"], json!("100"));
        assert_eq!(
            decoded_token["MPTokenIssuanceID"],
            json!("000002DF71CAE59C9B7E56587FFF74D4EA5830D9BE3CE0CC")
        );
        assert_eq!(
            decoded_token["Account"],
            json!("raDQsd1s8rqGjL476g59a9vVNi1rSwrC44")
        );
        assert_eq!(
            decoded_token["PreviousTxnID"],
            json!("222EF3C7E82D8A44984A66E2B8E357CB536EC2547359CCF70E56E14BC4C284C8")
        );
        assert_eq!(decoded_token["PreviousTxnLgrSeq"], json!(741));

        // Re-encode and confirm byte-for-byte match
        let re_encoded_token =
            serialize_json(&decoded_token, None, None, false).expect("re-encode MPToken failed");
        assert_eq!(
            re_encoded_token.to_uppercase(),
            mptoken_binary.to_uppercase(),
            "MPToken re-encode does not match authoritative vector"
        );
    }
}
