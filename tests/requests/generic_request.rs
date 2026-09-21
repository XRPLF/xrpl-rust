// Scenarios:
//   - ledger_accept: verify GenericRequest can drive a rippled command that
//     has no typed XRPLRequest variant (standalone-only admin RPC).
//   - server_state via GenericRequest: verify extra `params` fields flatten
//     correctly onto the wire alongside `command`, and the response comes back.

use crate::common::with_blockchain_lock;
use serde_json::{Map, Value};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::generic_request::GenericRequest;

/// `ledger_accept` is an admin-only standalone RPC that forces the local
/// rippled to close the current ledger. There is no typed request model for
/// it in xrpl-rust — this test proves `GenericRequest` covers the gap.
#[tokio::test]
async fn test_generic_request_ledger_accept() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        let request = GenericRequest::new("ledger_accept", None, Map::new());
        let response = client
            .request(request.into())
            .await
            .expect("ledger_accept request failed");

        // Response body carries the newly-closed ledger info. Serialize the
        // typed result back to JSON so we can inspect it without depending on
        // which XRPLResult variant it landed in (ledger_accept has no typed
        // model — it may match `Other` or a shape-compatible variant).
        let result = response.result.expect("ledger_accept returned no result");
        let value: Value =
            serde_json::to_value(&result).expect("failed to re-serialize ledger_accept result");
        assert!(
            value.get("ledger_hash").is_some() || value.get("ledger_current_index").is_some(),
            "ledger_accept result missing ledger_hash/ledger_current_index: {:?}",
            value
        );
    })
    .await;
}

/// `server_state` has a typed variant in xrpl-rust, but we route it through
/// `GenericRequest` here to prove that extra `params` (in this case the empty
/// object) serialize correctly, and to smoke-test a second command.
#[tokio::test]
async fn test_generic_request_server_state_with_params() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        // `server_state` accepts an optional `ledger_index` param; pass one
        // to exercise the params-flattening path.
        let mut params = Map::new();
        params.insert("ledger_index".into(), Value::String("validated".into()));
        let request = GenericRequest::new("server_state", Some("gr-1".into()), params);

        let response = client
            .request(request.into())
            .await
            .expect("server_state via GenericRequest failed");

        let result = response
            .result
            .expect("server_state via GenericRequest returned no result");
        // `server_state` has a typed result, but going through GenericRequest
        // we still get a typed `ServerState` variant back. Re-serialize to
        // inspect the raw JSON in a variant-agnostic way.
        let value: Value =
            serde_json::to_value(&result).expect("failed to re-serialize server_state result");
        assert!(
            value.get("state").is_some(),
            "server_state result missing `state` object: {:?}",
            value
        );
    })
    .await;
}
