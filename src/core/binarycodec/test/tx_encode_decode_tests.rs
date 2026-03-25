//! Round-trip tests (mirrors xrpl.js tx-encode-decode.test.ts)

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
