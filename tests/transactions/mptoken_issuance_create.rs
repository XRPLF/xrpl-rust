// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate
//
// Scenario:
//   - metadata_round_trip: encode XLS-89 metadata, submit an MPTokenIssuanceCreate,
//     read the issuance back from the ledger, and decode the on-ledger blob to
//     confirm it matches the original metadata.
//
// There is no SDK transaction model for MPTokenIssuanceCreate on `main`, so this
// test builds the transaction as raw JSON and submits it through the node's
// `submit` command (server-side signing with the wallet seed). This keeps the
// test focused on the XLS-89 encode/decode helpers without depending on any MPT
// transaction model.

use crate::common::{constants, generate_funded_wallet, ledger_accept, with_blockchain_lock};
use serde_json::{json, Value};
use xrpl::utils::mptoken_metadata::{
    decode_mptoken_metadata, encode_mptoken_metadata, MPTokenMetadata,
    MPTokenMetadataAdditionalInfo, MPTokenMetadataUri,
};

/// POST a raw JSON-RPC request to the standalone node and return its `result`.
async fn rpc(method: &str, params: Value) -> Value {
    let body = json!({ "method": method, "params": [params] });
    let response: Value = reqwest::Client::new()
        .post(constants::STANDALONE_URL)
        .json(&body)
        .send()
        .await
        .expect("rpc request failed")
        .json()
        .await
        .expect("rpc response was not valid JSON");
    response["result"].clone()
}

#[tokio::test]
async fn test_mptoken_issuance_create_metadata_round_trip() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let account = wallet.classic_address.clone();

        // Build XLS-89 metadata and encode it to the compact on-ledger blob.
        let metadata = MPTokenMetadata {
            ticker: "TBILL".into(),
            name: "T-Bill Yield Token".into(),
            desc: Some("A yield-bearing stablecoin backed by short-term U.S. Treasuries.".into()),
            icon: "https://example.org/tbill-icon.png".into(),
            asset_class: "rwa".into(),
            asset_subclass: Some("treasury".into()),
            issuer_name: "Example Yield Co.".into(),
            uris: Some(vec![MPTokenMetadataUri {
                uri: "https://exampleyield.co/tbill".into(),
                category: "website".into(),
                title: "Product Page".into(),
            }]),
            additional_info: Some(MPTokenMetadataAdditionalInfo::Object(
                json!({ "interest_rate": "5.00%", "maturity_date": "2045-06-30" })
                    .as_object()
                    .unwrap()
                    .clone(),
            )),
        };
        let encoded = encode_mptoken_metadata(&metadata).expect("encode metadata");

        // Submit a raw MPTokenIssuanceCreate transaction (the node signs it with
        // the wallet seed and autofills Fee/Sequence/SigningPubKey).
        let tx_json = json!({
            "TransactionType": "MPTokenIssuanceCreate",
            "Account": account,
            "AssetScale": 2,
            "MaximumAmount": "9223372036854775807",
            "MPTokenMetadata": encoded,
        });
        let submit = rpc(
            "submit",
            json!({ "tx_json": tx_json, "secret": wallet.seed }),
        )
        .await;
        assert_eq!(
            submit["engine_result"].as_str(),
            Some("tesSUCCESS"),
            "MPTokenIssuanceCreate did not succeed: {submit}"
        );
        ledger_accept().await;

        // Read the issuance back from the ledger and confirm the metadata round-trips.
        let result = rpc(
            "account_objects",
            json!({ "account": account, "ledger_index": "validated" }),
        )
        .await;
        let objects = result["account_objects"]
            .as_array()
            .expect("account_objects missing from response");
        let issuance = objects
            .iter()
            .find(|obj| obj["LedgerEntryType"] == "MPTokenIssuance")
            .expect("MPTokenIssuance object not found on ledger");
        let on_ledger_hex = issuance["MPTokenMetadata"]
            .as_str()
            .expect("MPTokenMetadata missing from issuance object");

        let decoded = decode_mptoken_metadata(on_ledger_hex).expect("decode metadata");
        let expected = serde_json::to_value(&metadata).expect("serialize metadata");
        assert_eq!(decoded, expected, "on-ledger metadata did not round-trip");
    })
    .await;
}
