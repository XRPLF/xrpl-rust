//! Happy-path integration tests for XLS-0096 ConfidentialMPT transactions
//! against the local standalone node.
//!
//! `ConfidentialMPTConvert` and `ConfidentialMPTMergeInbox` need real ledger
//! state — a confidential-capable `MPTokenIssuance` with a registered
//! `IssuerEncryptionKey`, an authorized holder, and a public MPT balance to
//! convert. xrpl-rust has no models for those prerequisite transactions yet,
//! so [`setup_confidential_issuance`] builds them with **raw JSON-RPC**
//! (`submit` with a `secret`, signed server-side — the node knows
//! `MPTokenIssuanceCreate`/`Set`/`Authorize` and `Payment` from MPTokensV1).
//! Only the transaction *under test* goes through the SDK.

use crate::common::constants::STANDALONE_URL;
use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::account::get_next_valid_seq_number;
use xrpl::asynch::clients::AsyncJsonRpcClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::mpt_crypto::{Privkey, Pubkey};
use xrpl::models::transactions::confidential_mpt_convert::ConfidentialMPTConvert;
use xrpl::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
use xrpl::wallet::Wallet;

// Fee for the prerequisite MPT transactions. None of these transactors
// override `calculateBaseFee`, so the node's 200-drop reference fee covers them.
const MPT_TXN_FEE: &str = "200";
// tfMPTCanConfidentialAmount — required for an issuance to support cMPT.
const TF_MPT_CAN_CONFIDENTIAL_AMOUNT: u32 = 0x0000_0080;

// ─────────────────────────────────────────────────────────────────────────
//  Raw JSON-RPC helpers (no SDK models needed for the prerequisites)
// ─────────────────────────────────────────────────────────────────────────

/// Uppercase-hex encode bytes for the transaction's hex string fields.
fn uppercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// Decode a hex string into bytes (e.g. the 24-byte MPTokenIssuanceID).
fn hex_to_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

/// POST a JSON-RPC request to the standalone node and return `result`.
async fn rpc(body: serde_json::Value) -> serde_json::Value {
    let resp: serde_json::Value = reqwest::Client::new()
        .post(STANDALONE_URL)
        .json(&body)
        .send()
        .await
        .expect("rpc request")
        .json()
        .await
        .expect("rpc json");
    resp["result"].clone()
}

/// Submit a server-signed `tx_json` (`secret` = wallet seed), assert
/// `tesSUCCESS`, and advance the ledger. Used only for prerequisite setup.
async fn submit_signed(seed: &str, tx_json: serde_json::Value) {
    let tx_type = tx_json["TransactionType"].as_str().unwrap_or("?").to_string();
    let result = rpc(serde_json::json!({
        "method": "submit",
        "params": [{ "secret": seed, "tx_json": tx_json }],
    }))
    .await;
    let code = result["engine_result"].as_str().unwrap_or("<none>");
    assert_eq!(
        code, "tesSUCCESS",
        "prerequisite {tx_type} failed: {code} — {}",
        result["engine_result_message"].as_str().unwrap_or("")
    );
    ledger_accept().await;
}

/// The `mpt_issuance_id` of the (single) issuance owned by `account`.
async fn sole_issuance_id(account: &str) -> String {
    let result = rpc(serde_json::json!({
        "method": "account_objects",
        "params": [{ "account": account, "type": "mpt_issuance", "ledger_index": "validated" }],
    }))
    .await;
    result["account_objects"][0]["mpt_issuance_id"]
        .as_str()
        .expect("mpt_issuance_id present")
        .to_string()
}

/// The holder's `MPToken` ledger object for `issuance_id`.
async fn holder_mptoken(holder: &str, issuance_id: &str) -> serde_json::Value {
    let result = rpc(serde_json::json!({
        "method": "account_objects",
        "params": [{ "account": holder, "type": "mptoken", "ledger_index": "validated" }],
    }))
    .await;
    result["account_objects"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|o| o["MPTokenIssuanceID"].as_str() == Some(issuance_id))
        .cloned()
        .expect("holder MPToken for issuance")
}

/// The holder's public (unencrypted) MPT balance for `issuance_id` (0 if absent).
async fn holder_public_mpt_balance(holder: &str, issuance_id: &str) -> u64 {
    let amount = holder_mptoken(holder, issuance_id).await["MPTAmount"].clone();
    amount
        .as_u64()
        .or_else(|| amount.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

/// Decrypt a hex-encoded 66-byte ElGamal confidential balance with the holder's
/// secret key, recovering the plaintext amount.
fn decrypt_confidential_balance(hex_ciphertext: &str, holder_sk: &Privkey) -> u64 {
    use xrpl::mpt_crypto::{encrypt, Ciphertext};
    let bytes: [u8; 66] = hex_to_bytes(hex_ciphertext)
        .try_into()
        .expect("66-byte ElGamal ciphertext");
    encrypt::decrypt(&Ciphertext::new(bytes), holder_sk).expect("decrypt confidential balance")
}

// ─────────────────────────────────────────────────────────────────────────
//  Prerequisite ledger state for a confidential MPT
// ─────────────────────────────────────────────────────────────────────────

/// Everything a holder needs to make a successful first `ConfidentialMPTConvert`:
/// a funded holder account that is authorized on a confidential-capable
/// issuance (whose `IssuerEncryptionKey` is `issuer_elgamal_pk`) and holds a
/// public MPT balance. The holder's ElGamal keypair is generated here too.
struct ConfidentialSetup {
    holder: Wallet,
    issuance_id: String,
    issuer_elgamal_pk: Pubkey,
    holder_elgamal_sk: Privkey,
    holder_elgamal_pk: Pubkey,
}

/// Build the prerequisite ledger state via raw JSON-RPC:
/// `MPTokenIssuanceCreate` → `MPTokenIssuanceSet` (register IssuerEncryptionKey)
/// → `MPTokenAuthorize` → `Payment` (public MPT balance to the holder).
async fn setup_confidential_issuance() -> ConfidentialSetup {
    use xrpl::mpt_crypto::keypair;

    // Issuer and holder are distinct funded accounts (Convert requires the
    // converting account != issuer). Their seeds let us sign server-side.
    let issuer = generate_funded_wallet().await;
    let holder = generate_funded_wallet().await;

    // ElGamal encryption keypairs (separate from the accounts' signing keys).
    let (_issuer_sk, issuer_elgamal_pk) = keypair::generate().expect("issuer ElGamal keypair");
    let (holder_elgamal_sk, holder_elgamal_pk) =
        keypair::generate().expect("holder ElGamal keypair");

    // 1. Issuer creates a confidential-capable issuance (no TransferFee allowed
    //    alongside the confidential flag).
    submit_signed(
        &issuer.seed,
        serde_json::json!({
            "TransactionType": "MPTokenIssuanceCreate",
            "Account": issuer.classic_address,
            "Flags": TF_MPT_CAN_CONFIDENTIAL_AMOUNT,
            "AssetScale": 0,
            "MaximumAmount": "1000000000",
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;
    let issuance_id = sole_issuance_id(&issuer.classic_address).await;

    // 2. Issuer registers its ElGamal public key on the issuance.
    submit_signed(
        &issuer.seed,
        serde_json::json!({
            "TransactionType": "MPTokenIssuanceSet",
            "Account": issuer.classic_address,
            "MPTokenIssuanceID": issuance_id,
            "IssuerEncryptionKey": uppercase_hex(issuer_elgamal_pk.as_bytes()),
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;

    // 3. Holder opts in to the issuance.
    submit_signed(
        &holder.seed,
        serde_json::json!({
            "TransactionType": "MPTokenAuthorize",
            "Account": holder.classic_address,
            "MPTokenIssuanceID": issuance_id,
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;

    // 4. Issuer sends the holder a public MPT balance to convert.
    submit_signed(
        &issuer.seed,
        serde_json::json!({
            "TransactionType": "Payment",
            "Account": issuer.classic_address,
            "Destination": holder.classic_address,
            "Amount": { "mpt_issuance_id": issuance_id, "value": "1000" },
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;

    ConfidentialSetup {
        holder,
        issuance_id,
        issuer_elgamal_pk,
        holder_elgamal_sk,
        holder_elgamal_pk,
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  ConfidentialMPTConvert crypto (real mpt-crypto material)
// ─────────────────────────────────────────────────────────────────────────

/// Hex-encoded crypto fields for a first (registration) `ConfidentialMPTConvert`.
struct CMPTConvertBundle {
    holder_encrypted_amount: String,
    issuer_encrypted_amount: String,
    blinding_factor: String,
    holder_encryption_key: String,
    zk_proof: String,
}

/// Build the Convert crypto bound to the real issuance + holder sequence.
///
/// The amount is encrypted under both the holder's key and the issuance's
/// **registered** `IssuerEncryptionKey` with the same revealed blinding factor;
/// the Schnorr `ZKProof` proves knowledge of the holder's secret key, bound via
/// the context hash to (holder account, issuance id, sequence) — exactly what
/// rippled's preclaim verifies.
fn build_convert_material(
    holder_account: &str,
    issuance_id_hex: &str,
    sequence: u32,
    amount: u64,
    issuer_elgamal_pk: &Pubkey,
    holder_elgamal_sk: &Privkey,
    holder_elgamal_pk: &Pubkey,
) -> CMPTConvertBundle {
    use xrpl::core::addresscodec::decode_classic_address;
    use xrpl::mpt_crypto::{context, encrypt, prove, AccountId, IssuanceId};

    let r = encrypt::random_blinding_factor().expect("blinding factor");
    let holder_ct = encrypt::encrypt(amount, holder_elgamal_pk, &r).expect("holder ciphertext");
    let issuer_ct = encrypt::encrypt(amount, issuer_elgamal_pk, &r).expect("issuer ciphertext");

    let account: [u8; 20] = decode_classic_address(holder_account)
        .expect("decode classic address")
        .try_into()
        .expect("20-byte AccountID");
    let issuance: [u8; 24] = hex_to_bytes(issuance_id_hex)
        .try_into()
        .expect("24-byte MPTokenIssuanceID");
    let ctx = context::convert(&AccountId::new(account), &IssuanceId::new(issuance), sequence)
        .expect("convert context hash");
    let proof =
        prove::convert(holder_elgamal_sk, holder_elgamal_pk, &ctx).expect("convert proof");

    CMPTConvertBundle {
        holder_encrypted_amount: uppercase_hex(holder_ct.as_bytes()),
        issuer_encrypted_amount: uppercase_hex(issuer_ct.as_bytes()),
        blinding_factor: uppercase_hex(r.as_bytes()),
        holder_encryption_key: uppercase_hex(holder_elgamal_pk.as_bytes()),
        zk_proof: uppercase_hex(proof.as_bytes()),
    }
}

/// Submit the holder's first `ConfidentialMPTConvert` via the SDK and assert
/// `tesSUCCESS`. Shared by both happy-path tests (MergeInbox needs a prior
/// Convert to initialize the holder's confidential balances).
async fn convert_public_to_confidential(
    setup: &ConfidentialSetup,
    client: &AsyncJsonRpcClient,
    amount: u64,
) {
    // The proof binds to the exact sequence the transaction carries.
    let sequence =
        get_next_valid_seq_number(setup.holder.classic_address.clone().into(), client, None)
            .await
            .expect("fetch holder sequence");
    let m = build_convert_material(
        &setup.holder.classic_address,
        &setup.issuance_id,
        sequence,
        amount,
        &setup.issuer_elgamal_pk,
        &setup.holder_elgamal_sk,
        &setup.holder_elgamal_pk,
    );

    let mut tx = ConfidentialMPTConvert::new(
        setup.holder.classic_address.clone().into(),
        None,           // account_txn_id
        None,           // fee — autofilled (cMPT = 10× base)
        None,           // last_ledger_sequence
        None,           // memos
        Some(sequence), // bound into the proof context above
        None,           // signers
        None,           // source_tag
        None,           // ticket_sequence
        setup.issuance_id.clone().into(),
        amount.to_string().into(),
        m.holder_encrypted_amount.into(),
        m.issuer_encrypted_amount.into(),
        m.blinding_factor.into(),
        Some(m.holder_encryption_key.into()), // first Convert registers the key
        None,                                 // AuditorEncryptedAmount (no auditor)
        Some(m.zk_proof.into()),              // first Convert carries the proof
    );

    let resp = sign_and_submit(&mut tx, client, &setup.holder, true, true)
        .await
        .expect("convert sign_and_submit");
    assert_eq!(
        resp.engine_result, "tesSUCCESS",
        "ConfidentialMPTConvert: {} — {}",
        resp.engine_result, resp.engine_result_message
    );
    ledger_accept().await;
}

// ─────────────────────────────────────────────────────────────────────────
//  Happy-path tests
// ─────────────────────────────────────────────────────────────────────────

/// First (registration) `ConfidentialMPTConvert`: converts part of the holder's
/// public MPT balance into confidential form. SDK-built; expects `tesSUCCESS`
/// and verifies the public balance is debited by exactly the converted amount.
#[tokio::test]
async fn conf_mpt_convert_transaction() {
    with_blockchain_lock(|| async {
        let setup = setup_confidential_issuance().await;
        let client = get_client().await;

        let amount: u64 = 100;
        let before =
            holder_public_mpt_balance(&setup.holder.classic_address, &setup.issuance_id).await;
        convert_public_to_confidential(&setup, client, amount).await;
        let after =
            holder_public_mpt_balance(&setup.holder.classic_address, &setup.issuance_id).await;
        assert_eq!(
            after,
            before - amount,
            "Convert should debit {amount} from the public balance ({before} → {after})"
        );

        // The holder's confidential state is now initialized: the ElGamal key is
        // registered, and the encrypted inbox decrypts (with the holder's secret
        // key) to exactly the converted amount.
        let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
        let registered_key = uppercase_hex(setup.holder_elgamal_pk.as_bytes());
        assert_eq!(
            mptoken["HolderEncryptionKey"].as_str(),
            Some(registered_key.as_str()),
            "HolderEncryptionKey should be the registered holder key"
        );
        let inbox = decrypt_confidential_balance(
            mptoken["ConfidentialBalanceInbox"]
                .as_str()
                .expect("ConfidentialBalanceInbox present"),
            &setup.holder_elgamal_sk,
        );
        assert_eq!(inbox, amount, "confidential inbox should decrypt to {amount}");
    })
    .await;
}

/// `ConfidentialMPTMergeInbox` after a Convert: the Convert deposits into the
/// holder's confidential inbox, then MergeInbox folds it into the spending
/// balance.
#[tokio::test]
async fn merge_inbox_jsonrpc_round_trip() {
    with_blockchain_lock(|| async {
        let setup = setup_confidential_issuance().await;
        let client = get_client().await;

        // Initialize the holder's confidential state + populate the inbox.
        let amount: u64 = 100;
        convert_public_to_confidential(&setup, client, amount).await;

        let mut tx = ConfidentialMPTMergeInbox::new(
            setup.holder.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee — autofilled (cMPT = 10× base)
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            setup.issuance_id.clone().into(),
        );

        let resp = sign_and_submit(&mut tx, client, &setup.holder, true, true)
            .await
            .expect("merge_inbox sign_and_submit");
        assert_eq!(
            resp.engine_result, "tesSUCCESS",
            "ConfidentialMPTMergeInbox: {} — {}",
            resp.engine_result, resp.engine_result_message
        );
        ledger_accept().await;

        // After merge, the inbox is folded into the spending balance and reset:
        // spending decrypts to the converted amount, inbox to zero.
        let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
        let spending = decrypt_confidential_balance(
            mptoken["ConfidentialBalanceSpending"]
                .as_str()
                .expect("ConfidentialBalanceSpending present"),
            &setup.holder_elgamal_sk,
        );
        let inbox = decrypt_confidential_balance(
            mptoken["ConfidentialBalanceInbox"]
                .as_str()
                .expect("ConfidentialBalanceInbox present"),
            &setup.holder_elgamal_sk,
        );
        assert_eq!(spending, amount, "spending should decrypt to {amount} after merge");
        assert_eq!(inbox, 0, "inbox should decrypt to 0 (reset) after merge");
        // The first Convert leaves the version at 0 (omitted); MergeInbox bumps it.
        assert_eq!(
            mptoken["ConfidentialBalanceVersion"].as_u64(),
            Some(1),
            "merge should bump ConfidentialBalanceVersion to 1"
        );
    })
    .await;
}
