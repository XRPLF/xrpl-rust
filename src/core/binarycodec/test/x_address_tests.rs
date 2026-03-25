//! X-Address tests (mirrors xrpl.js x-address.test.ts)

use super::*;
use crate::core::binarycodec::test_cases::load_x_codec_fixtures;

#[test]
fn test_x_codec_fixtures_transactions() {
    let fixtures = load_x_codec_fixtures();
    let total = fixtures.transactions.len();
    println!("\n=== x-codec-fixtures transactions ({}) ===", total);

    let mut passed = 0;
    let mut failed = 0;

    for (i, entry) in fixtures.transactions.iter().enumerate() {
        // Test 1: encode(xjson) == encode(rjson)
        let encoded_r = match encode(&entry.rjson) {
            Ok(e) => e,
            Err(e) => {
                println!("  ✗ x-codec[{}] encode(rjson) FAILED: {:?}", i, e);
                failed += 1;
                continue;
            }
        };
        let encoded_x = match encode(&entry.xjson) {
            Ok(e) => e,
            Err(e) => {
                println!("  ✗ x-codec[{}] encode(xjson) FAILED: {:?}", i, e);
                failed += 1;
                continue;
            }
        };
        if encoded_r.to_uppercase() != encoded_x.to_uppercase() {
            println!("  ✗ x-codec[{}] encode mismatch", i);
            println!("    encode(rjson): {}", encoded_r);
            println!("    encode(xjson): {}", encoded_x);
            failed += 1;
            continue;
        }

        // Test 2: decode(encode(xjson)) == rjson
        match decode(&encoded_x) {
            Ok(decoded) => {
                if decoded == entry.rjson {
                    println!("  ✓ x-codec[{}] passed", i);
                    passed += 1;
                } else {
                    println!("  ✗ x-codec[{}] decode mismatch", i);
                    println!("    Expected (rjson): {}", entry.rjson);
                    println!("    Got (decoded):    {}", decoded);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ x-codec[{}] decode FAILED: {:?}", i, e);
                failed += 1;
            }
        }
    }

    println!(
        "  === x-codec transactions: {} passed, {} failed out of {} ===",
        passed, failed, total
    );

    if failed > 0 {
        panic!(
            "x-codec transactions: {} out of {} tests failed",
            failed, total
        );
    }
}

#[test]
fn test_x_address_account_with_tag() {
    // X-Address Account is equivalent to a classic address w/ SourceTag
    let json_x = serde_json::json!({
        "OwnerCount": 0,
        "Account": "XVXdn5wEVm5G4UhEHWDPqjvdeH361P7BsapL4m2D2XnPSwT",
        "PreviousTxnLgrSeq": 7,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "DF530FB14C5304852F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    let json_r = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "PreviousTxnLgrSeq": 7,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "DF530FB14C5304852F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000",
        "SourceTag": 12345
    });

    let encoded_x = encode(&json_x).expect("encode x failed");
    let encoded_r = encode(&json_r).expect("encode r failed");
    assert_eq!(
        encoded_x, encoded_r,
        "X-address encode should match classic+tag"
    );

    let decoded = decode(&encoded_x).expect("decode failed");
    assert_eq!(
        decoded, json_r,
        "decoded X-address should include SourceTag"
    );
}

#[test]
fn test_x_address_issuer_no_tag() {
    // Encoding issuer X-Address w/ undefined destination tag is OK
    let json_x = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Destination": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Issuer": "XVXdn5wEVm5G4UhEHWDPqjvdeH361P4GETfNyyXGaoqBj71",
        "PreviousTxnLgrSeq": 7,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "DF530FB14C5304852F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    let json_r = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Destination": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Issuer": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "PreviousTxnLgrSeq": 7,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "DF530FB14C5304852F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });

    assert_eq!(
        encode(&json_x).expect("encode x"),
        encode(&json_r).expect("encode r"),
        "Issuer X-Address w/ no tag should match classic"
    );
}

#[test]
fn test_x_address_issued_currency() {
    // Encodes issued currency w/ x-address
    let json_x = serde_json::json!({
        "TakerPays": {
            "currency": "USD",
            "issuer": "X7WZKEeNVS2p9Tire9DtNFkzWBZbFtJHWxDjN9fCrBGqVA4",
            "value": "7072.8"
        }
    });
    let json_r = serde_json::json!({
        "TakerPays": {
            "currency": "USD",
            "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
            "value": "7072.8"
        }
    });

    assert_eq!(
        encode(&json_x).expect("encode x"),
        encode(&json_r).expect("encode r"),
        "Issued currency X-address should match classic"
    );
}

#[test]
fn test_x_address_issuer_with_tag_throws() {
    // X-Address with tag throws for Issuer field
    let json = serde_json::json!({
        "OwnerCount": 0,
        "Account": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Destination": "rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv",
        "Issuer": "XVXdn5wEVm5G4UhEHWDPqjvdeH361P7BsapL4m2D2XnPSwT",
        "PreviousTxnLgrSeq": 7,
        "LedgerEntryType": "AccountRoot",
        "PreviousTxnID": "DF530FB14C5304852F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF",
        "Flags": 0,
        "Sequence": 1,
        "Balance": "10000000000"
    });
    assert!(encode(&json).is_err(), "Issuer with tag should error");
}

#[test]
fn test_x_address_issued_currency_with_tag_throws() {
    // Issued currency issuer with tag throws
    let json = serde_json::json!({
        "TakerPays": {
            "currency": "USD",
            "issuer": "X7WZKEeNVS2p9Tire9DtNFkzWBZbFtSiS2eDBib7svZXuc2",
            "value": "7072.8"
        }
    });
    assert!(
        encode(&json).is_err(),
        "Issued currency with tagged issuer should error"
    );
}
