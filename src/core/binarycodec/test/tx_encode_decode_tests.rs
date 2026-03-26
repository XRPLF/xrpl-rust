//! Round-trip tests

use super::*;

#[test]
fn test_roundtrip_encode_decode_no_amount_fee() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 1,
        "TransactionType": "Payment"
    });
    let encoded = encode(&tx).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(tx, decoded);
}

#[test]
fn test_roundtrip_encode_decode_with_amount_fee() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 1,
        "TransactionType": "Payment",
        "Amount": "1000",
        "Fee": "10"
    });
    let encoded = encode(&tx).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(tx, decoded);
}

#[test]
fn test_roundtrip_encode_decode_with_ticket_count() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 1,
        "TransactionType": "Payment",
        "TicketCount": 2
    });
    let encoded = encode(&tx).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(tx, decoded);
}

#[test]
fn test_roundtrip_encode_decode_with_ticket_sequence() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 0,
        "TransactionType": "Payment",
        "TicketSequence": 2
    });
    let encoded = encode(&tx).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(tx, decoded);
}

#[test]
fn test_roundtrip_issued_currency_xrp_hex() {
    // An issued currency with hex currency code that looks like XRP
    let tx = serde_json::json!({
        "TransactionType": "TrustSet",
        "Flags": 0,
        "Sequence": 19,
        "LimitAmount": {
            "value": "200",
            "currency": "0000000000000000000000005852500000000000",
            "issuer": "r9hEDb4xBGRfBCcX3E4FirDWQBAYtpxC8K"
        },
        "Fee": "10",
        "SigningPubKey": "023076CBB7A61837F1A23D4A3DD7CE810B694992EB0959AB9D6F4BB6FED6F8CC26",
        "TxnSignature": "304502202D0CD77D8E765E3783C309CD663723B18406B7950A348A6F301492916A990FC70221008A76D586111205304F10ADEFDFDDAF804EF202D8CD1E492DC6E1AA8030EA1844",
        "Account": "rPtfQWdcdhuL9eNeNv5YfmekSX3K7vJHbG"
    });
    let encoded = encode(&tx).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(tx, decoded);
}

#[test]
fn test_encode_invalid_amount_throws() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 1,
        "TransactionType": "Payment",
        "Amount": "1000.001",
        "Fee": "10"
    });
    assert!(encode(&tx).is_err());
}

#[test]
fn test_encode_invalid_fee_throws() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "Flags": 2147483648u64,
        "Sequence": 1,
        "TransactionType": "Payment",
        "Amount": "1000",
        "Fee": "10.123"
    });
    assert!(encode(&tx).is_err());
}

// ── BASE10 UInt64 field tests ─────────────

const MPT_ISSUANCE_ENTRY_BINARY: &str =
    "11007E220000006224000002DF25000002E434000000000000000030187FFFFFFFFFFFFFFF30190000000000000064552E78C1FFBDDAEE077253CEB12CFEA83689AA0899F94762190A357208DADC76FE701EC1EC7B226E616D65223A2255532054726561737572792042696C6C20546F6B656E222C2273796D626F6C223A225553544254222C22646563696D616C73223A322C22746F74616C537570706C79223A313030303030302C22697373756572223A225553205472656173757279222C22697373756544617465223A22323032342D30332D3235222C226D6174757269747944617465223A22323032352D30332D3235222C226661636556616C7565223A2231303030222C22696E74657265737452617465223A22322E35222C22696E7465726573744672657175656E6379223A22517561727465726C79222C22636F6C6C61746572616C223A22555320476F7665726E6D656E74222C226A7572697364696374696F6E223A22556E6974656420537461746573222C22726567756C61746F7279436F6D706C69616E6365223A2253454320526567756C6174696F6E73222C22736563757269747954797065223A2254726561737572792042696C6C222C2265787465726E616C5F75726C223A2268747470733A2F2F6578616D706C652E636F6D2F742D62696C6C2D746F6B656E2D6D657461646174612E6A736F6E227D8414A4D893CFBC4DC6AE877EB585F90A3B47528B958D051003";

const MPTOKEN_ENTRY_BINARY: &str =
    "11007F220000000025000002E5340000000000000000301A000000000000006455222EF3C7E82D8A44984A66E2B8E357CB536EC2547359CCF70E56E14BC4C284C881143930DB9A74C26D96CB58ADFFD7E8BB78BCFE62340115000002DF71CAE59C9B7E56587FFF74D4EA5830D9BE3CE0CC";

#[test]
fn test_base10_uint64_mpt_issuance_encode() {
    let json = serde_json::json!({
        "AssetScale": 3,
        "Flags": 98,
        "Issuer": "rGpdGXDV2RFPeLEfWS9RFo5Nh9cpVDToZa",
        "LedgerEntryType": "MPTokenIssuance",
        "MPTokenMetadata": "7B226E616D65223A2255532054726561737572792042696C6C20546F6B656E222C2273796D626F6C223A225553544254222C22646563696D616C73223A322C22746F74616C537570706C79223A313030303030302C22697373756572223A225553205472656173757279222C22697373756544617465223A22323032342D30332D3235222C226D6174757269747944617465223A22323032352D30332D3235222C226661636556616C7565223A2231303030222C22696E74657265737452617465223A22322E35222C22696E7465726573744672657175656E6379223A22517561727465726C79222C22636F6C6C61746572616C223A22555320476F7665726E6D656E74222C226A7572697364696374696F6E223A22556E6974656420537461746573222C22726567756C61746F7279436F6D706C69616E6365223A2253454320526567756C6174696F6E73222C22736563757269747954797065223A2254726561737572792042696C6C222C2265787465726E616C5F75726C223A2268747470733A2F2F6578616D706C652E636F6D2F742D62696C6C2D746F6B656E2D6D657461646174612E6A736F6E227D",
        "MaximumAmount": "9223372036854775807",
        "OutstandingAmount": "100",
        "OwnerNode": "0000000000000000",
        "PreviousTxnID": "2E78C1FFBDDAEE077253CEB12CFEA83689AA0899F94762190A357208DADC76FE",
        "PreviousTxnLgrSeq": 740,
        "Sequence": 735
    });
    let encoded = encode(&json).expect("encode failed");
    assert_eq!(encoded, MPT_ISSUANCE_ENTRY_BINARY);
}

#[test]
fn test_base10_uint64_mpt_issuance_decode() {
    let decoded = decode(MPT_ISSUANCE_ENTRY_BINARY).expect("decode failed");
    assert_eq!(decoded["MaximumAmount"], "9223372036854775807");
    assert_eq!(decoded["OutstandingAmount"], "100");
}

#[test]
fn test_base10_uint64_mpt_issuance_roundtrip() {
    let json = serde_json::json!({
        "AssetScale": 3,
        "Flags": 98,
        "Issuer": "rGpdGXDV2RFPeLEfWS9RFo5Nh9cpVDToZa",
        "LedgerEntryType": "MPTokenIssuance",
        "MPTokenMetadata": "7B226E616D65223A2255532054726561737572792042696C6C20546F6B656E222C2273796D626F6C223A225553544254222C22646563696D616C73223A322C22746F74616C537570706C79223A313030303030302C22697373756572223A225553205472656173757279222C22697373756544617465223A22323032342D30332D3235222C226D6174757269747944617465223A22323032352D30332D3235222C226661636556616C7565223A2231303030222C22696E74657265737452617465223A22322E35222C22696E7465726573744672657175656E6379223A22517561727465726C79222C22636F6C6C61746572616C223A22555320476F7665726E6D656E74222C226A7572697364696374696F6E223A22556E6974656420537461746573222C22726567756C61746F7279436F6D706C69616E6365223A2253454320526567756C6174696F6E73222C22736563757269747954797065223A2254726561737572792042696C6C222C2265787465726E616C5F75726C223A2268747470733A2F2F6578616D706C652E636F6D2F742D62696C6C2D746F6B656E2D6D657461646174612E6A736F6E227D",
        "MaximumAmount": "9223372036854775807",
        "OutstandingAmount": "100",
        "OwnerNode": "0000000000000000",
        "PreviousTxnID": "2E78C1FFBDDAEE077253CEB12CFEA83689AA0899F94762190A357208DADC76FE",
        "PreviousTxnLgrSeq": 740,
        "Sequence": 735
    });
    let encoded = encode(&json).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(json, decoded);
}

#[test]
fn test_base10_uint64_mptoken_encode() {
    let json = serde_json::json!({
        "Account": "raDQsd1s8rqGjL476g59a9vVNi1rSwrC44",
        "Flags": 0,
        "LedgerEntryType": "MPToken",
        "MPTAmount": "100",
        "MPTokenIssuanceID": "000002DF71CAE59C9B7E56587FFF74D4EA5830D9BE3CE0CC",
        "OwnerNode": "0000000000000000",
        "PreviousTxnID": "222EF3C7E82D8A44984A66E2B8E357CB536EC2547359CCF70E56E14BC4C284C8",
        "PreviousTxnLgrSeq": 741
    });
    let encoded = encode(&json).expect("encode failed");
    assert_eq!(encoded, MPTOKEN_ENTRY_BINARY);
}

#[test]
fn test_base10_uint64_mptoken_decode() {
    let decoded = decode(MPTOKEN_ENTRY_BINARY).expect("decode failed");
    assert_eq!(decoded["MPTAmount"], "100");
}

#[test]
fn test_base10_uint64_mptoken_roundtrip() {
    let json = serde_json::json!({
        "Account": "raDQsd1s8rqGjL476g59a9vVNi1rSwrC44",
        "Flags": 0,
        "LedgerEntryType": "MPToken",
        "MPTAmount": "100",
        "MPTokenIssuanceID": "000002DF71CAE59C9B7E56587FFF74D4EA5830D9BE3CE0CC",
        "OwnerNode": "0000000000000000",
        "PreviousTxnID": "222EF3C7E82D8A44984A66E2B8E357CB536EC2547359CCF70E56E14BC4C284C8",
        "PreviousTxnLgrSeq": 741
    });
    let encoded = encode(&json).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(json, decoded);
}

// ── Lowercase hex tests ─────────────

const LOWERCASE_HEX_STR: &str =
    "1100612200000000240000000125000068652D0000000055B6632D6376A2D9319F20A1C6DCCB486432D1E4A79951229D4C3DE2946F51D56662400009184E72A00081140DD319918CD5AE792BF7EC80D63B0F01B4573BBC";

const LOWERCASE_HEX_BIN: &str =
    "1100612200000000240000000125000000082D00000000550735A0B32B2A3F4C938B76D6933003E29447DB8C7CE382BBE089402FF12A03E56240000002540BE400811479927BAFFD3D04A26096C0C97B1B0D45B01AD3C0";

#[test]
fn test_lowercase_hex_correctly_decodes() {
    // xrpl.js: decode(lower) == decode(str)
    let lower = LOWERCASE_HEX_STR.to_lowercase();
    let decoded_lower = decode(&lower).expect("decode lowercase failed");
    let decoded_upper = decode(LOWERCASE_HEX_STR).expect("decode uppercase failed");
    assert_eq!(decoded_lower, decoded_upper);
}

#[test]
fn test_lowercase_hex_re_encodes_to_uppercase() {
    // xrpl.js: encode(decode(lower)) == str
    let lower = LOWERCASE_HEX_STR.to_lowercase();
    let decoded = decode(&lower).expect("decode failed");
    let re_encoded = encode(&decoded).expect("encode failed");
    assert_eq!(re_encoded, LOWERCASE_HEX_STR);
}

#[test]
fn test_lowercase_hex_encode_when_hex_field_lowercase() {
    // xrpl.js: encode(json) == bin, where json has lowercase PreviousTxnID
    let json = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rUnFEsHjxqTswbivzL2DNHBb34rhAgZZZK",
        "PreviousTxnLgrSeq": 8,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "0735a0b32b2a3f4c938b76d6933003e29447db8c7ce382bbe089402ff12a03e5",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    let encoded = encode(&json).expect("encode failed");
    assert_eq!(encoded, LOWERCASE_HEX_BIN);
}

#[test]
fn test_lowercase_hex_re_decodes_to_uppercase() {
    // xrpl.js: decode(encode(json)) == jsonUpper (PreviousTxnID uppercased)
    let json = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rUnFEsHjxqTswbivzL2DNHBb34rhAgZZZK",
        "PreviousTxnLgrSeq": 8,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "0735a0b32b2a3f4c938b76d6933003e29447db8c7ce382bbe089402ff12a03e5",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    let json_upper = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rUnFEsHjxqTswbivzL2DNHBb34rhAgZZZK",
        "PreviousTxnLgrSeq": 8,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "0735A0B32B2A3F4C938B76D6933003E29447DB8C7CE382BBE089402FF12A03E5",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    let encoded = encode(&json).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(decoded, json_upper);
}

// ============================================================
// UInt range checking tests (mirrors xrpl.js uint.test.ts)
// ============================================================

#[test]
fn test_uint8_range_overflow() {
    // 300 > u8::MAX (255) should error, not silently truncate
    // AssetScale is a UInt8 field
    let tx = serde_json::json!({
        "Flags": 0,
        "Issuer": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "AssetScale": 300u64,
        "LedgerEntryType": "MPTokenIssuance"
    });
    assert!(encode(&tx).is_err());
}

#[test]
fn test_uint8_max_valid() {
    let tx = serde_json::json!({
        "Flags": 0,
        "Issuer": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "AssetScale": 255,
        "LedgerEntryType": "MPTokenIssuance"
    });
    assert!(encode(&tx).is_ok());
}

#[test]
fn test_uint32_range_overflow() {
    // 5000000000 > u32::MAX (4294967295) should error
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "TransactionType": "Payment",
        "Sequence": 5000000000u64
    });
    assert!(encode(&tx).is_err());
}

// ============================================================
// Int32 tests (mirrors xrpl.js int.test.ts)
// ============================================================

#[test]
fn test_int32_encode_decode_negative() {
    // Loan object with negative LoanScale — mirrors xrpl.js int.test.ts
    let loan = serde_json::json!({
        "Borrower": "rs5fUokF7Y5bxNkstM4p4JYHgqzYkFamCg",
        "GracePeriod": 60,
        "LoanBrokerID": "18F91BD8009DAF09B5E4663BE7A395F5F193D0657B12F8D1E781EB3D449E8151",
        "LoanScale": -11,
        "LoanSequence": 1,
        "NextPaymentDueDate": 822779431,
        "PaymentInterval": 400,
        "PaymentRemaining": 1,
        "PeriodicPayment": "10000",
        "PrincipalOutstanding": "10000",
        "StartDate": 822779031,
        "TotalValueOutstanding": "10000",
        "LedgerEntryType": "Loan"
    });
    let encoded = encode(&loan).expect("encode negative LoanScale failed");
    let decoded = decode(&encoded).expect("decode negative LoanScale failed");
    assert_eq!(decoded, loan);
}

#[test]
fn test_int32_encode_decode_positive() {
    let loan = serde_json::json!({
        "Borrower": "rs5fUokF7Y5bxNkstM4p4JYHgqzYkFamCg",
        "GracePeriod": 60,
        "LoanBrokerID": "18F91BD8009DAF09B5E4663BE7A395F5F193D0657B12F8D1E781EB3D449E8151",
        "LoanScale": 5,
        "LoanSequence": 1,
        "NextPaymentDueDate": 822779431,
        "PaymentInterval": 400,
        "PaymentRemaining": 1,
        "PeriodicPayment": "10000",
        "PrincipalOutstanding": "10000",
        "StartDate": 822779031,
        "TotalValueOutstanding": "10000",
        "LedgerEntryType": "Loan"
    });
    let encoded = encode(&loan).expect("encode positive LoanScale failed");
    let decoded = decode(&encoded).expect("decode positive LoanScale failed");
    assert_eq!(decoded, loan);
}

// ============================================================
// Pseudo-transaction ACCOUNT_ZERO tests (mirrors xrpl.js pseudo-transaction.test.ts)
// ============================================================

#[test]
fn test_pseudo_transaction_encode() {
    let json = serde_json::json!({
        "Account": "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
        "Sequence": 0,
        "Fee": "0",
        "SigningPubKey": "",
        "Signature": ""
    });
    let expected_binary =
        "24000000006840000000000000007300760081140000000000000000000000000000000000000000";
    let encoded = encode(&json).expect("encode pseudo-transaction failed");
    assert_eq!(encoded, expected_binary);
}

#[test]
fn test_pseudo_transaction_roundtrip() {
    let json = serde_json::json!({
        "Account": "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
        "Sequence": 0,
        "Fee": "0",
        "SigningPubKey": "",
        "Signature": ""
    });
    let encoded = encode(&json).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(decoded, json);
}

#[test]
fn test_blank_account_is_account_zero() {
    let json_blank = serde_json::json!({
        "Account": "",
        "Sequence": 0,
        "Fee": "0",
        "SigningPubKey": "",
        "Signature": ""
    });
    let expected_binary =
        "24000000006840000000000000007300760081140000000000000000000000000000000000000000";
    let encoded = encode(&json_blank).expect("encode blank account failed");
    assert_eq!(encoded, expected_binary);
}

#[test]
fn test_blank_account_decodes_to_account_zero() {
    let json_blank = serde_json::json!({
        "Account": "",
        "Sequence": 0,
        "Fee": "0",
        "SigningPubKey": "",
        "Signature": ""
    });
    let json_expected = serde_json::json!({
        "Account": "rrrrrrrrrrrrrrrrrrrrrhoLvTp",
        "Sequence": 0,
        "Fee": "0",
        "SigningPubKey": "",
        "Signature": ""
    });
    let encoded = encode(&json_blank).expect("encode failed");
    let decoded = decode(&encoded).expect("decode failed");
    assert_eq!(decoded, json_expected);
}

// ============================================================
// Unknown field rejection test (mirrors xrpl.js tx-encode-decode.test.ts)
// ============================================================

#[test]
fn test_throws_on_unknown_field() {
    let tx = serde_json::json!({
        "Account": "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ",
        "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "TransactionType": "Payment",
        "Sequence": 1,
        "BadField": 1
    });
    assert!(encode(&tx).is_err());
}
