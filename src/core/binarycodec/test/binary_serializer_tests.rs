//! Binary Serializer tests (mirrors xrpl.js binary-serializer.test.ts)

use super::*;
use crate::core::binarycodec::test_cases::load_whole_objects;

/// Helper: assert encode(tx_json) == expected_binary
fn assert_encode_equals(name: &str, tx_json: &Value, expected_binary: &str) {
    let encoded = encode(tx_json).unwrap_or_else(|e| panic!("{} encode failed: {:?}", name, e));
    assert_eq!(
        encoded.to_uppercase(),
        expected_binary.to_uppercase(),
        "{} encode mismatch",
        name
    );
}

#[test]
fn test_delivermin() {
    let tx: Value = serde_json::from_str(include_str!("../test_data/delivermin-tx.json")).unwrap();
    let binary: String =
        serde_json::from_str(include_str!("../test_data/delivermin-tx-binary.json")).unwrap();
    assert_encode_equals("DeliverMin", &tx, &binary);
}

#[test]
fn test_signerlistset() {
    let tx: Value =
        serde_json::from_str(include_str!("../test_data/signerlistset-tx.json")).unwrap();
    let binary: String =
        serde_json::from_str(include_str!("../test_data/signerlistset-tx-binary.json")).unwrap();
    let meta_binary: String = serde_json::from_str(include_str!(
        "../test_data/signerlistset-tx-meta-binary.json"
    ))
    .unwrap();
    assert_encode_equals("SignerListSet", &tx, &binary);
    let meta = tx.get("meta").expect("signerlistset-tx.json missing meta");
    assert_encode_equals("SignerListSet meta", meta, &meta_binary);
}

#[test]
fn test_deposit_preauth() {
    let tx: Value =
        serde_json::from_str(include_str!("../test_data/deposit-preauth-tx.json")).unwrap();
    let binary: String =
        serde_json::from_str(include_str!("../test_data/deposit-preauth-tx-binary.json")).unwrap();
    let meta_binary: String = serde_json::from_str(include_str!(
        "../test_data/deposit-preauth-tx-meta-binary.json"
    ))
    .unwrap();
    assert_encode_equals("DepositPreauth", &tx, &binary);
    let meta = tx
        .get("meta")
        .expect("deposit-preauth-tx.json missing meta");
    assert_encode_equals("DepositPreauth meta", meta, &meta_binary);
}

#[test]
fn test_escrow() {
    let create_tx: Value =
        serde_json::from_str(include_str!("../test_data/escrow-create-tx.json")).unwrap();
    let create_binary: String =
        serde_json::from_str(include_str!("../test_data/escrow-create-binary.json")).unwrap();
    assert_encode_equals("EscrowCreate", &create_tx, &create_binary);

    let finish_tx: Value =
        serde_json::from_str(include_str!("../test_data/escrow-finish-tx.json")).unwrap();
    let finish_binary: String =
        serde_json::from_str(include_str!("../test_data/escrow-finish-binary.json")).unwrap();
    let finish_meta_binary: String =
        serde_json::from_str(include_str!("../test_data/escrow-finish-meta-binary.json")).unwrap();
    assert_encode_equals("EscrowFinish", &finish_tx, &finish_binary);
    let finish_meta = finish_tx
        .get("meta")
        .expect("escrow-finish-tx.json missing meta");
    assert_encode_equals("EscrowFinish meta", finish_meta, &finish_meta_binary);

    let cancel_tx: Value =
        serde_json::from_str(include_str!("../test_data/escrow-cancel-tx.json")).unwrap();
    let cancel_binary: String =
        serde_json::from_str(include_str!("../test_data/escrow-cancel-binary.json")).unwrap();
    assert_encode_equals("EscrowCancel", &cancel_tx, &cancel_binary);
}

#[test]
fn test_payment_channel() {
    let create_tx: Value =
        serde_json::from_str(include_str!("../test_data/payment-channel-create-tx.json")).unwrap();
    let create_binary: String = serde_json::from_str(include_str!(
        "../test_data/payment-channel-create-binary.json"
    ))
    .unwrap();
    assert_encode_equals("PaymentChannelCreate", &create_tx, &create_binary);

    let fund_tx: Value =
        serde_json::from_str(include_str!("../test_data/payment-channel-fund-tx.json")).unwrap();
    let fund_binary: String = serde_json::from_str(include_str!(
        "../test_data/payment-channel-fund-binary.json"
    ))
    .unwrap();
    assert_encode_equals("PaymentChannelFund", &fund_tx, &fund_binary);

    let claim_tx: Value =
        serde_json::from_str(include_str!("../test_data/payment-channel-claim-tx.json")).unwrap();
    let claim_binary: String = serde_json::from_str(include_str!(
        "../test_data/payment-channel-claim-binary.json"
    ))
    .unwrap();
    assert_encode_equals("PaymentChannelClaim", &claim_tx, &claim_binary);
}

#[test]
fn test_negative_unl() {
    let fixture: Value =
        serde_json::from_str(include_str!("../test_data/negative-unl.json")).unwrap();
    let tx = fixture.get("tx").expect("negative-unl.json missing tx");
    let binary = fixture
        .get("binary")
        .expect("negative-unl.json missing binary")
        .as_str()
        .unwrap();
    assert_encode_equals("NegativeUNL", tx, binary);
    let decoded = decode(binary).expect("NegativeUNL decode failed");
    assert_eq!(decoded, *tx, "NegativeUNL decode mismatch");
}

#[test]
fn test_ticket_create() {
    let tx: Value =
        serde_json::from_str(include_str!("../test_data/ticket-create-tx.json")).unwrap();
    let binary: String =
        serde_json::from_str(include_str!("../test_data/ticket-create-binary.json")).unwrap();
    assert_encode_equals("TicketCreate", &tx, &binary);
}

#[test]
fn test_nf_token() {
    let fixtures: Value = serde_json::from_str(include_str!("../test_data/nf-token.json")).unwrap();
    let obj = fixtures
        .as_object()
        .expect("nf-token.json is not an object");
    for (tx_name, entry) in obj {
        let tx_json = &entry["tx"]["json"];
        let tx_binary = entry["tx"]["binary"].as_str().unwrap();
        assert_encode_equals(
            &alloc::format!("NFToken {} tx", tx_name),
            tx_json,
            tx_binary,
        );
        let decoded = decode(tx_binary)
            .unwrap_or_else(|e| panic!("NFToken {} tx decode failed: {:?}", tx_name, e));
        assert_eq!(decoded, *tx_json, "NFToken {} tx decode mismatch", tx_name);

        let meta_json = &entry["meta"]["json"];
        let meta_binary = entry["meta"]["binary"].as_str().unwrap();
        assert_encode_equals(
            &alloc::format!("NFToken {} meta", tx_name),
            meta_json,
            meta_binary,
        );
        let decoded_meta = decode(meta_binary)
            .unwrap_or_else(|e| panic!("NFToken {} meta decode failed: {:?}", tx_name, e));
        assert_eq!(
            decoded_meta, *meta_json,
            "NFToken {} meta decode mismatch",
            tx_name
        );
    }
}

#[test]
fn test_whole_objects_recycle() {
    let whole_objects = load_whole_objects();
    let total = whole_objects.len();

    println!("\n=== Running {} whole_objects recycle tests ===\n", total);

    let mut passed = 0;
    let mut failed = 0;

    for (i, wo) in whole_objects.iter().enumerate() {
        let blob = &wo.blob_with_no_signing;
        match decode(blob) {
            Ok(decoded) => match encode(&decoded) {
                Ok(re_encoded) => {
                    if re_encoded.to_uppercase() == blob.to_uppercase() {
                        println!("  ✓ whole_objects[{}] passed", i);
                        passed += 1;
                    } else {
                        println!("  ✗ whole_objects[{}] recycle MISMATCH", i);
                        println!("    Original:   {}", blob);
                        println!("    Re-encoded: {}", re_encoded);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ whole_objects[{}] re-encode FAILED: {:?}", i, e);
                    failed += 1;
                }
            },
            Err(e) => {
                println!("  ✗ whole_objects[{}] decode FAILED: {:?}", i, e);
                failed += 1;
            }
        }
    }

    println!(
        "\n=== Whole Objects: {} passed, {} failed out of {} ===\n",
        passed, failed, total
    );

    if failed > 0 {
        panic!(
            "whole_objects: {} out of {} recycle tests failed",
            failed, total
        );
    }
}
