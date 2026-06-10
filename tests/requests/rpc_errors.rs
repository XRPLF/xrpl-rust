use crate::common::constants::STANDALONE_URL;
use serde_json::{json, Value};
use xrpl::models::results::{XRPLResponse, XRPLRpcError};

const UNKNOWN_COMMAND_METHOD: &str = "not_a_real_command";

#[tokio::test]
async fn test_rpc_error_mapping_from_xrpld_response() {
    let body: Value = reqwest::Client::new()
        .post(STANDALONE_URL)
        .json(&json!({
            // Intentionally bypass RequestMethod here: the test needs xrpld to return
            // a server-side unknown-command response so we can verify XRPLRpcError mapping.
            "method": UNKNOWN_COMMAND_METHOD,
            "params": [{}]
        }))
        .send()
        .await
        .expect("unknown-command request failed")
        .json()
        .await
        .expect("unknown-command response JSON parse failed");

    let response: XRPLResponse<'_> = serde_json::from_value(body["result"].clone())
        .expect("unknown-command result should deserialize as XRPLResponse");

    assert_eq!(response.rpc_error(), Some(XRPLRpcError::UnknownCommand));
}
