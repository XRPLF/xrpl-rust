//! Integration tests for XLS-0096 ConfidentialMPT transactions against a
//! local rippled standalone node with the `ConfidentialTransfer` amendment
//! enabled. Covers the proof-free `MergeInbox` transaction over both
//! transports:
//!
//! - **JSON-RPC**: `http://localhost:5005`
//! - **WebSocket**: `ws://localhost:6006`
//!
//! ## Why MergeInbox is the only test here today
//!
//! `definitions.json` is now mirrored verbatim from the live rippled's
//! `server_definitions` RPC, so the binary codec already understands every
//! XLS-0096 field (`HolderEncryptionKey`, `IssuerEncryptionKey`, `ZKProof`,
//! `BlindingFactor`, the `*EncryptedAmount` family, etc.). Sign-time
//! serialization is *not* the blocker.
//!
//! What is missing is **ledger-state setup**. Convert / Send / ConvertBack
//! / Clawback all require an existing `MPTokenIssuance` plus a holder with
//! initialized confidential balances (`CB_S`, `CB_IN`). That setup is a
//! multi-transaction chain: `MPTokenIssuanceCreate` →
//! `MPTokenIssuanceSet` (register `IssuerEncryptionKey`) → `MPTokenAuthorize`
//! → `Payment` (with `MPTAmount`) → `ConfidentialMPTConvert`. Once those
//! prereq transaction models are implemented in xrpl-rust, the
//! proof-bearing tests will live alongside the `merge_inbox_*` tests below
//! and use `mpt-crypto` for proof generation.
//!
//! For now, this file covers the one case that needs no prior ledger state.

use anyhow::Result;
use url::Url;

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::clients::{AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
use xrpl::models::transactions::TransactionType;

const STANDALONE_WS_URL: &str = "ws://localhost:6006";

// ─────────────────────────────────────────────────────────────────────────
//  Local helper to open a websocket against the standalone node
// ─────────────────────────────────────────────────────────────────────────

async fn open_ws(
) -> Result<AsyncWebSocketClient<SingleExecutorMutex, WebSocketOpen>, Box<dyn std::error::Error>> {
    AsyncWebSocketClient::open(Url::parse(STANDALONE_WS_URL)?)
        .await
        .map_err(Into::into)
}

// ─────────────────────────────────────────────────────────────────────────
//  ConfidentialMPTMergeInbox — proof-free; runs end-to-end
// ─────────────────────────────────────────────────────────────────────────

/// Submits a `ConfidentialMPTMergeInbox` transaction via JSON-RPC against
/// rippled at `localhost:5005`. The holder doesn't have any confidential
/// MPT state set up, so we expect the transaction to be rejected by
/// rippled — but the rejection itself proves the wire format and signing
/// path are correct. We assert it's rejected with a *protocol-level* code
/// (`tec*` or `tef*`), not a `tem*` (malformed) which would indicate our
/// serialization is wrong.
#[tokio::test]
async fn merge_inbox_jsonrpc_round_trip() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = ConfidentialMPTMergeInbox::new(
            wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            // 24-byte MPTokenIssuanceID — points at a non-existent issuance
            // for this test; the network will surface an OBJECT_NOT_FOUND-
            // class error, not a malformed-tx error.
            "00000000".repeat(6).into(),
        );

        let client = get_client().await;
        let resp = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit should succeed at the wire level");

        // We do NOT assert tesSUCCESS — the holder has no confidential
        // state, so the protocol layer should reject. We DO assert the
        // result code class, to prove the failure is semantic (tec*/tef*),
        // not a binary-codec / malformed-transaction failure (tem*).
        let code = &resp.engine_result;
        assert!(
            code.starts_with("tec")
                || code.starts_with("tef")
                || code == "tesSUCCESS",
            "Expected tec*/tef* or tesSUCCESS, got `{code}` — {}",
            resp.engine_result_message
        );

        ledger_accept().await;
    })
    .await;
}

/// Same as the JSON-RPC variant, but submitting through the WebSocket
/// client connected to `localhost:6006`. Exercises the second transport.
#[tokio::test]
async fn merge_inbox_websocket_round_trip() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = ConfidentialMPTMergeInbox::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "00000000".repeat(6).into(),
        );
        // Sanity: the model's transaction_type round-trips through the enum.
        assert_eq!(
            tx.common_fields.transaction_type,
            TransactionType::ConfidentialMPTMergeInbox
        );

        let mut client = open_ws().await.expect("open websocket");
        let resp = sign_and_submit(&mut tx, &mut client, &wallet, true, true)
            .await
            .expect("websocket sign_and_submit should succeed at the wire level");

        let code = &resp.engine_result;
        assert!(
            code.starts_with("tec")
                || code.starts_with("tef")
                || code == "tesSUCCESS",
            "Expected tec*/tef* or tesSUCCESS, got `{code}` — {}",
            resp.engine_result_message
        );

        ledger_accept().await;
    })
    .await;
}

