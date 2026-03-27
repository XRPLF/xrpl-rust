//! Codec-fixtures tests

use super::*;
use crate::core::binarycodec::test_cases::{self, load_codec_fixtures};

/// Helper: run encode+decode tests for a list of codec-fixture entries.
fn run_codec_fixture_suite(name: &str, entries: &[test_cases::CodecFixtureEntry]) {
    let total = entries.len();
    let mut passed = 0;
    let mut failed = 0;

    for (i, entry) in entries.iter().enumerate() {
        // Test encode
        let encode_result = encode(&entry.json);
        match &encode_result {
            Ok(encoded) => {
                if encoded.to_uppercase() != entry.binary.to_uppercase() {
                    println!(
                        "  ✗ {}[{}] encode MISMATCH\n    Expected: {}\n    Got:      {}",
                        name, i, entry.binary, encoded
                    );
                    failed += 1;
                    continue;
                }
            }
            Err(e) => {
                println!("  ✗ {}[{}] encode FAILED: {:?}", name, i, e);
                failed += 1;
                continue;
            }
        }

        // Test decode
        let decode_result = decode(&entry.binary);
        match decode_result {
            Ok(decoded) => {
                if decoded == entry.json {
                    passed += 1;
                } else {
                    println!("  ✗ {}[{}] decode MISMATCH", name, i);
                    println!("    Expected: {}", entry.json);
                    println!("    Got:      {}", decoded);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ {}[{}] decode FAILED: {:?}", name, i, e);
                failed += 1;
            }
        }
    }

    println!(
        "  === {}: {} passed, {} failed out of {} ===",
        name, passed, failed, total
    );

    if failed > 0 {
        panic!("{}: {} out of {} tests failed", name, failed, total);
    }
}

#[test]
fn test_codec_fixtures_transactions() {
    let fixtures = load_codec_fixtures();
    println!(
        "\n=== codec-fixtures transactions ({}) ===",
        fixtures.transactions.len()
    );
    run_codec_fixture_suite("transactions", &fixtures.transactions);
}

#[test]
fn test_codec_fixtures_account_state() {
    let fixtures = load_codec_fixtures();
    println!(
        "\n=== codec-fixtures accountState ({}) ===",
        fixtures.account_state.len()
    );
    run_codec_fixture_suite("accountState", &fixtures.account_state);
}

#[test]
fn test_codec_fixtures_ledger_data() {
    let fixtures = load_codec_fixtures();
    println!(
        "\n=== codec-fixtures ledgerData ({}) ===",
        fixtures.ledger_data.len()
    );

    let mut failed = 0;
    for (i, entry) in fixtures.ledger_data.iter().enumerate() {
        match decode_ledger_data(&entry.binary) {
            Ok(decoded) => {
                if decoded == entry.json {
                    println!("  ✓ ledgerData[{}] passed", i);
                } else {
                    println!("  ✗ ledgerData[{}] decode MISMATCH", i);
                    println!("    Expected: {}", entry.json);
                    println!("    Got:      {}", decoded);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ ledgerData[{}] decode FAILED: {:?}", i, e);
                failed += 1;
            }
        }
    }

    if failed > 0 {
        panic!(
            "ledgerData: {} out of {} tests failed",
            failed,
            fixtures.ledger_data.len()
        );
    }
}
