use anyhow::Result;
use url::Url;

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::account::get_next_valid_seq_number;
use xrpl::asynch::clients::{AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::transactions::confidential_mpt_convert::ConfidentialMPTConvert;
use xrpl::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
use xrpl::models::XRPAmount;

// ─────────────────────────────────────────────────────────────────────────
//  ConfidentialMPTMergeInbox — proof-free; runs end-to-end
// ─────────────────────────────────────────────────────────────────────────

/// MergeInbox over JSON-RPC. The holder has no confidential state, so a
/// semantic rejection (`tec*`/`tef*`, not malformed `tem*`) confirms the wire
/// format and signing path are correct.
#[tokio::test]
async fn merge_inbox_jsonrpc_round_trip() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = ConfidentialMPTMergeInbox::new(
            wallet.classic_address.clone().into(),
            None,                            // account_txn_id
            Some(XRPAmount::from("100000")), // fee — cMPT requires elevated fee
            None,                            // last_ledger_sequence
            None,                            // memos
            None,                            // sequence
            None,                            // signers
            None,                            // source_tag
            None,                            // ticket_sequence
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
            code == "tesSUCCESS",
            "Expected tesSUCCESS, got `{code}` — {}",
            resp.engine_result_message
        );

        ledger_accept().await;
    })
    .await;
}

// ─────────────────────────────────────────────────────────────────────────
//  ConfidentialMPTConvert — proof-bearing; real mpt-crypto material
//
//  These build genuine XLS-0096 crypto with the re-exported `xrpl::mpt_crypto`
//  crate: an ElGamal holder keypair, the revealed blinding factor, the
//  holder/issuer ElGamal ciphertexts of the amount, and — on the first
//  (registration) Convert — the 33-byte holder key plus the 64-byte Schnorr
//  Proof of Knowledge. The proof is bound to (account, issuance, sequence), so
//  the test fetches the holder's Sequence first and submits with that value.
//
//  The MPTokenIssuanceID is a placeholder (no MPTokenIssuance exists until the
//  prerequisite MPToken* models land — roadblock #2), so these assert the
//  node's response is *semantic* (`tec*`/`tef*`) rather than *malformed*
//  (`tem*`): i.e. the node parsed valid cMPT crypto and reasoned about it,
//  failing only because the issuance is absent. A `tesSUCCESS` happy path
//  additionally needs that prerequisite chain.
// ─────────────────────────────────────────────────────────────────────────

/// 24-byte placeholder MPTokenIssuanceID — no such issuance exists on-ledger.
const PLACEHOLDER_ISSUANCE_ID: [u8; 24] = [0x11; 24];

/// Uppercase-hex encode bytes for the transaction's hex string fields.
fn uppercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// Hex-encoded Convert fields from `xrpl::mpt_crypto`. `holder_encryption_key`
/// and `zk_proof` apply only to the first (registration) Convert.
struct CMPTConvertBundle {
    issuance_id: String,
    holder_encrypted_amount: String,
    issuer_encrypted_amount: String,
    blinding_factor: String,
    holder_encryption_key: String,
    zk_proof: String,
}

/// Builds the hex-encoded crypto fields for a `ConfidentialMPTConvert`.
///
/// `holder_*`/`issuer_*` are ElGamal encryption keypairs (secp256k1), separate
/// from the account's signing key. `holder_elgamal_pk` is registered as the
/// `HolderEncryptionKey` and proven by the `ZKProof` (bound to account +
/// issuance + sequence); `issuer_elgamal_pk` mirrors the amount for the issuer
/// (throwaway — its secret is unused here).
fn build_convert_material(holder_account: &str, sequence: u32, amount: u64) -> CMPTConvertBundle {
    use xrpl::core::addresscodec::decode_classic_address;
    use xrpl::mpt_crypto::{context, encrypt, keypair, prove, AccountId, IssuanceId};

    // ElGamal encryption keypairs (secp256k1) — NOT the account's signing key.
    // Holder: pk becomes the registered HolderEncryptionKey; sk decrypts the
    // confidential balance. Issuer: stands in for the issuance's
    // IssuerEncryptionKey (its secret is unused, so the keypair is throwaway).
    let (holder_elgamal_sk, holder_elgamal_pk) =
        keypair::generate().expect("holder ElGamal keypair");
    let (_issuer_elgamal_sk, issuer_elgamal_pk) =
        keypair::generate().expect("issuer ElGamal keypair");

    // Shared ElGamal randomness `r`, revealed as BlindingFactor so validators
    // can deterministically verify the ciphertexts. The amount is encrypted once
    // under each party's ElGamal public key.
    let r = encrypt::random_blinding_factor().expect("blinding factor");
    let holder_amount_ct =
        encrypt::encrypt(amount, &holder_elgamal_pk, &r).expect("holder ciphertext");
    let issuer_amount_ct =
        encrypt::encrypt(amount, &issuer_elgamal_pk, &r).expect("issuer ciphertext");

    // Context binds the Schnorr PoK to (account, issuance, sequence); these
    // must match the submitted transaction exactly.
    let account: [u8; 20] = decode_classic_address(holder_account)
        .expect("decode classic address")
        .try_into()
        .expect("20-byte AccountID");
    let ctx = context::convert(
        &AccountId::new(account),
        &IssuanceId::new(PLACEHOLDER_ISSUANCE_ID),
        sequence,
    )
    .expect("convert context hash");
    // Proves knowledge of holder_elgamal_sk for the published holder_elgamal_pk.
    let proof =
        prove::convert(&holder_elgamal_sk, &holder_elgamal_pk, &ctx).expect("convert proof");

    CMPTConvertBundle {
        issuance_id: uppercase_hex(&PLACEHOLDER_ISSUANCE_ID),
        holder_encrypted_amount: uppercase_hex(holder_amount_ct.as_bytes()),
        issuer_encrypted_amount: uppercase_hex(issuer_amount_ct.as_bytes()),
        blinding_factor: uppercase_hex(r.as_bytes()),
        holder_encryption_key: uppercase_hex(holder_elgamal_pk.as_bytes()),
        zk_proof: uppercase_hex(proof.as_bytes()),
    }
}

/// First (registration) `ConfidentialMPTConvert` over JSON-RPC: carries
/// `HolderEncryptionKey` + `ZKProof` from `xrpl::mpt_crypto`.
#[tokio::test]
async fn conf_mpt_convert_transaction() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let client = get_client().await;

        // Bind the proof to the exact Sequence the transaction will carry.
        let sequence =
            get_next_valid_seq_number(wallet.classic_address.clone().into(), client, None)
                .await
                .expect("fetch holder sequence");
        let m = build_convert_material(&wallet.classic_address, sequence, 1000);

        let mut tx = ConfidentialMPTConvert::new(
            wallet.classic_address.clone().into(),
            None,                            // account_txn_id
            Some(XRPAmount::from("100000")), // fee — cMPT proof verification is priced above base
            None,                            // last_ledger_sequence
            None,                            // memos
            Some(sequence),                  // bound into the proof context above
            None,                            // signers
            None,                            // source_tag
            None,                            // ticket_sequence
            m.issuance_id.into(),
            "1000".into(), // MPTAmount (public amount being converted)
            m.holder_encrypted_amount.into(),
            m.issuer_encrypted_amount.into(),
            m.blinding_factor.into(),
            Some(m.holder_encryption_key.into()), // present on first Convert
            None,                                 // AuditorEncryptedAmount (no auditor)
            Some(m.zk_proof.into()),              // present on first Convert
        );

        let resp = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit should succeed at the wire level");

        let code = &resp.engine_result;
        assert!(
            code == "tesSUCCESS",
            "Expected tesSUCCESS, \
             got `{code}` — {}",
            resp.engine_result_message
        );

        ledger_accept().await;
    })
    .await;
}
