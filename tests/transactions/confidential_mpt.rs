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
use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::account::get_next_valid_seq_number;
use xrpl::asynch::clients::AsyncJsonRpcClient;
use xrpl::mpt_crypto::{Privkey, Pubkey};
use xrpl::models::transactions::confidential_mpt_clawback::ConfidentialMPTClawback;
use xrpl::models::transactions::confidential_mpt_convert::ConfidentialMPTConvert;
use xrpl::models::transactions::confidential_mpt_convert_back::ConfidentialMPTConvertBack;
use xrpl::models::transactions::confidential_mpt_merge_inbox::ConfidentialMPTMergeInbox;
use xrpl::models::transactions::confidential_mpt_send::ConfidentialMPTSend;
use xrpl::wallet::Wallet;

// Fee for the prerequisite MPT transactions. None of these transactors
// override `calculateBaseFee`, so the node's 200-drop reference fee covers them.
const MPT_TXN_FEE: &str = "200";
// Public MPT balance the issuer funds each holder with (something to convert).
const FUNDED_MPT: &str = "1000";
// MPTokenIssuance create-time flags
const TF_MPT_CAN_TRANSFER: u32 = 0x0000_0020;
const TF_MPT_CAN_CLAWBACK: u32 = 0x0000_0040;
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

/// Decode an XRPL classic address into its 20-byte AccountID.
fn account_id_bytes(address: &str) -> [u8; 20] {
    xrpl::core::addresscodec::decode_classic_address(address)
        .expect("decode classic address")
        .try_into()
        .expect("20-byte AccountID")
}

/// Decode a hex `MPTokenIssuanceID` into its 24 bytes.
fn issuance_id_bytes(issuance_id: &str) -> [u8; 24] {
    hex_to_bytes(issuance_id)
        .try_into()
        .expect("24-byte MPTokenIssuanceID")
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

/// The holder's on-ledger spending ciphertext (`ConfidentialBalanceSpending`,
/// 66 bytes) and confidential balance version — the inputs a ConvertBack/Send
/// proof must bind to.
async fn onledger_spending(setup: &ConfidentialSetup) -> ([u8; 66], u32) {
    let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
    let spending: [u8; 66] = hex_to_bytes(
        mptoken["ConfidentialBalanceSpending"]
            .as_str()
            .expect("ConfidentialBalanceSpending present"),
    )
    .try_into()
    .expect("66-byte spending ciphertext");
    let version = mptoken["ConfidentialBalanceVersion"].as_u64().unwrap_or(0) as u32;
    (spending, version)
}

// ─────────────────────────────────────────────────────────────────────────
//  Prerequisite ledger state for a confidential MPT
// ─────────────────────────────────────────────────────────────────────────

/// Everything a holder needs to make a successful first `ConfidentialMPTConvert`:
/// a funded holder account that is authorized on a confidential-capable
/// issuance (whose `IssuerEncryptionKey` is `issuer_elgamal_pk`) and holds a
/// public MPT balance. The holder's ElGamal keypair is generated here too.
struct ConfidentialSetup {
    issuer: Wallet,
    holder: Wallet,
    issuance_id: String,
    issuer_elgamal_sk: Privkey,
    issuer_elgamal_pk: Pubkey,
    holder_elgamal_sk: Privkey,
    holder_elgamal_pk: Pubkey,
}

/// Build the prerequisite ledger state via raw JSON-RPC:
/// `MPTokenIssuanceCreate` → `MPTokenIssuanceSet` (register IssuerEncryptionKey)
/// → `MPTokenAuthorize` → `Payment` (public MPT balance to the holder).
async fn setup_confidential_issuance(issuance_flags: u32) -> ConfidentialSetup {
    use xrpl::mpt_crypto::keypair;

    // Issuer and holder are distinct funded accounts (Convert requires the
    // converting account != issuer). Their seeds let us sign server-side.
    let issuer = generate_funded_wallet().await;
    let holder = generate_funded_wallet().await;

    // ElGamal encryption keypairs (separate from the accounts' signing keys).
    let (issuer_elgamal_sk, issuer_elgamal_pk) =
        keypair::generate().expect("issuer ElGamal keypair");
    let (holder_elgamal_sk, holder_elgamal_pk) =
        keypair::generate().expect("holder ElGamal keypair");

    // 1. Issuer creates the issuance with the requested capabilities (caller
    //    sets the flags; no TransferFee allowed alongside the confidential one).
    submit_signed(
        &issuer.seed,
        serde_json::json!({
            "TransactionType": "MPTokenIssuanceCreate",
            "Account": issuer.classic_address,
            "Flags": issuance_flags,
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
            "Amount": { "mpt_issuance_id": issuance_id, "value": FUNDED_MPT },
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;

    ConfidentialSetup {
        issuer,
        holder,
        issuance_id,
        issuer_elgamal_sk,
        issuer_elgamal_pk,
        holder_elgamal_sk,
        holder_elgamal_pk,
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  ConfidentialMPTConvert crypto (real mpt-crypto material)
// ─────────────────────────────────────────────────────────────────────────

/// Hex-encoded crypto fields for a first (registration) `ConfidentialMPTConvert`.
struct ConfidentialMPTConvertBundle {
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
) -> ConfidentialMPTConvertBundle {
    use xrpl::mpt_crypto::{context, encrypt, prove, AccountId, IssuanceId};

    let r = encrypt::random_blinding_factor().expect("blinding factor");
    let holder_ct = encrypt::encrypt(amount, holder_elgamal_pk, &r).expect("holder ciphertext");
    let issuer_ct = encrypt::encrypt(amount, issuer_elgamal_pk, &r).expect("issuer ciphertext");

    let account = account_id_bytes(holder_account);
    let issuance = issuance_id_bytes(issuance_id_hex);
    let ctx = context::convert(&AccountId::new(account), &IssuanceId::new(issuance), sequence)
        .expect("convert context hash");
    let proof =
        prove::convert(holder_elgamal_sk, holder_elgamal_pk, &ctx).expect("convert proof");

    ConfidentialMPTConvertBundle {
        holder_encrypted_amount: uppercase_hex(holder_ct.as_bytes()),
        issuer_encrypted_amount: uppercase_hex(issuer_ct.as_bytes()),
        blinding_factor: uppercase_hex(r.as_bytes()),
        holder_encryption_key: uppercase_hex(holder_elgamal_pk.as_bytes()),
        zk_proof: uppercase_hex(proof.as_bytes()),
    }
}

/// Submit a first (registration) `ConfidentialMPTConvert` for `wallet` and
/// assert `tesSUCCESS`. Generic over the holder so it serves both the primary
/// holder and a Send destination.
#[allow(clippy::too_many_arguments)]
async fn submit_first_convert(
    client: &AsyncJsonRpcClient,
    wallet: &Wallet,
    holder_sk: &Privkey,
    holder_pk: &Pubkey,
    issuer_pk: &Pubkey,
    issuance_id: &str,
    amount: u64,
) {
    // The proof binds to the exact sequence the transaction carries.
    let sequence = get_next_valid_seq_number(wallet.classic_address.clone().into(), client, None)
        .await
        .expect("fetch holder sequence");
    let m = build_convert_material(
        &wallet.classic_address,
        issuance_id,
        sequence,
        amount,
        issuer_pk,
        holder_sk,
        holder_pk,
    );

    let mut tx = ConfidentialMPTConvert::new(
        wallet.classic_address.clone().into(),
        None,           // account_txn_id
        None,           // fee — autofilled (cMPT = 10× base)
        None,           // last_ledger_sequence
        None,           // memos
        Some(sequence), // bound into the proof context above
        None,           // signers
        None,           // source_tag
        None,           // ticket_sequence
        issuance_id.to_string().into(),
        amount.to_string().into(),
        m.holder_encrypted_amount.into(),
        m.issuer_encrypted_amount.into(),
        m.blinding_factor.into(),
        Some(m.holder_encryption_key.into()), // first Convert registers the key
        None,                                 // AuditorEncryptedAmount (no auditor)
        Some(m.zk_proof.into()),              // first Convert carries the proof
    );

    test_transaction(&mut tx, wallet).await;
}

/// The primary holder's first Convert (the common case).
async fn convert_public_to_confidential(
    setup: &ConfidentialSetup,
    client: &AsyncJsonRpcClient,
    amount: u64,
) {
    submit_first_convert(
        client,
        &setup.holder,
        &setup.holder_elgamal_sk,
        &setup.holder_elgamal_pk,
        &setup.issuer_elgamal_pk,
        &setup.issuance_id,
        amount,
    )
    .await;
}

// ─────────────────────────────────────────────────────────────────────────
//  Happy-path tests
// ─────────────────────────────────────────────────────────────────────────

/// First (registration) `ConfidentialMPTConvert`: converts part of the holder's
/// public MPT balance into confidential form. SDK-built; expects `tesSUCCESS`
/// and verifies the public balance is debited by exactly the converted amount.
#[tokio::test]
async fn confidential_mpt_convert() {
    with_blockchain_lock(|| async {
        let setup = setup_confidential_issuance(TF_MPT_CAN_CONFIDENTIAL_AMOUNT).await;
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
async fn confidential_mpt_merge_inbox() {
    with_blockchain_lock(|| async {
        let setup = setup_confidential_issuance(TF_MPT_CAN_CONFIDENTIAL_AMOUNT).await;
        let client = get_client().await;

        // Initialize the holder's confidential state + populate the inbox.
        let amount: u64 = 100;
        convert_public_to_confidential(&setup, client, amount).await;
        merge_confidential_inbox(&setup).await;

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

/// `ConfidentialMPTClawback`: the issuer claws back the holder's confidential
/// balance. After a Convert the issuer's mirror of the holder's balance equals
/// the converted amount; the issuer reveals it, proves the mirror ciphertext
/// decrypts to it, and the holder's confidential balances are zeroed.
#[tokio::test]
async fn confidential_mpt_clawback() {
    with_blockchain_lock(|| async {
        let setup =
            setup_confidential_issuance(TF_MPT_CAN_CONFIDENTIAL_AMOUNT | TF_MPT_CAN_CLAWBACK).await;
        let client = get_client().await;

        let amount: u64 = 100;
        // Seed the holder's confidential balance + the issuer's mirror of it.
        convert_public_to_confidential(&setup, client, amount).await;

        // The clawback proof binds to (issuer, issuance, sequence, holder); set
        // the issuer's sequence explicitly so it matches the proof context.
        let sequence =
            get_next_valid_seq_number(setup.issuer.classic_address.clone().into(), client, None)
                .await
                .expect("fetch issuer sequence");
        let zk_proof = build_clawback_proof(&setup, sequence, amount).await;

        let mut tx = ConfidentialMPTClawback::new(
            setup.issuer.classic_address.clone().into(),
            None,           // account_txn_id
            None,           // fee — autofilled (cMPT = 10× base)
            None,           // last_ledger_sequence
            None,           // memos
            Some(sequence), // bound into the proof context above
            None,           // signers
            None,           // source_tag
            None,           // ticket_sequence
            setup.holder.classic_address.clone().into(),
            setup.issuance_id.clone().into(),
            amount.to_string().into(),
            zk_proof.into(),
        );

        test_transaction(&mut tx, &setup.issuer).await;

        // The holder's confidential balances are zeroed (decrypt to 0).
        let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
        let inbox = decrypt_confidential_balance(
            mptoken["ConfidentialBalanceInbox"]
                .as_str()
                .expect("ConfidentialBalanceInbox present"),
            &setup.holder_elgamal_sk,
        );
        assert_eq!(inbox, 0, "clawback should zero the holder's confidential inbox");
    })
    .await;
}

/// Build the issuer's 64-byte Clawback proof: reveals `amount` and proves the
/// issuer's mirror of the holder's balance (`IssuerEncryptedBalance`, read from
/// the ledger) decrypts to it under the issuer's ElGamal key.
async fn build_clawback_proof(setup: &ConfidentialSetup, sequence: u32, amount: u64) -> String {
    use xrpl::mpt_crypto::{context, prove, AccountId, Ciphertext, IssuanceId};

    let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
    let issuer_amount_mirror: [u8; 66] = hex_to_bytes(
        mptoken["IssuerEncryptedBalance"]
            .as_str()
            .expect("IssuerEncryptedBalance present"),
    )
    .try_into()
    .expect("66-byte issuer-mirror ciphertext");

    let issuer_account = account_id_bytes(&setup.issuer.classic_address);
    let holder_account = account_id_bytes(&setup.holder.classic_address);
    let issuance = issuance_id_bytes(&setup.issuance_id);

    let ctx = context::clawback(
        &AccountId::new(issuer_account),
        &IssuanceId::new(issuance),
        sequence,
        &AccountId::new(holder_account),
    )
    .expect("clawback context hash");
    let proof = prove::clawback(
        &setup.issuer_elgamal_sk,
        &setup.issuer_elgamal_pk,
        &ctx,
        amount,
        &Ciphertext::new(issuer_amount_mirror),
    )
    .expect("clawback proof");
    uppercase_hex(proof.as_bytes())
}

/// Submit the holder's `ConfidentialMPTMergeInbox` via the SDK and assert
/// `tesSUCCESS`. Moves the confidential inbox into the spending balance.
async fn merge_confidential_inbox(setup: &ConfidentialSetup) {
    let mut tx = ConfidentialMPTMergeInbox::new(
        setup.holder.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        setup.issuance_id.clone().into(),
    );
    test_transaction(&mut tx, &setup.holder).await;
}

/// `ConfidentialMPTConvertBack`: the holder withdraws confidential balance back
/// to public. Requires a *spending* balance (Convert + MergeInbox), then proves
/// (balance − amount) ≥ 0 against the on-ledger spending ciphertext. SDK-built;
/// expects `tesSUCCESS` and verifies the public credit + zeroed spending balance.
#[tokio::test]
async fn confidential_mpt_convert_back() {
    with_blockchain_lock(|| async {
        let setup = setup_confidential_issuance(TF_MPT_CAN_CONFIDENTIAL_AMOUNT).await;
        let client = get_client().await;

        let amount: u64 = 100;
        // Convert + merge so the full amount sits in the spending balance.
        convert_public_to_confidential(&setup, client, amount).await;
        merge_confidential_inbox(&setup).await;

        let public_before =
            holder_public_mpt_balance(&setup.holder.classic_address, &setup.issuance_id).await;

        // Withdraw the entire spending balance back to public.
        let sequence =
            get_next_valid_seq_number(setup.holder.classic_address.clone().into(), client, None)
                .await
                .expect("fetch holder sequence");
        let m = build_convert_back_material(&setup, sequence, amount, amount).await;

        let mut tx = ConfidentialMPTConvertBack::new(
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
            m.balance_commitment.into(),
            m.zk_proof.into(),
            None, // auditor_encrypted_amount (no auditor)
        );

        test_transaction(&mut tx, &setup.holder).await;

        // Public balance credited by the withdrawn amount; spending → 0.
        let public_after =
            holder_public_mpt_balance(&setup.holder.classic_address, &setup.issuance_id).await;
        assert_eq!(
            public_after,
            public_before + amount,
            "ConvertBack should credit {amount} to the public balance ({public_before} → {public_after})"
        );
        let mptoken = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
        let spending = decrypt_confidential_balance(
            mptoken["ConfidentialBalanceSpending"]
                .as_str()
                .expect("ConfidentialBalanceSpending present"),
            &setup.holder_elgamal_sk,
        );
        assert_eq!(spending, 0, "spending should decrypt to 0 after converting it all back");
    })
    .await;
}

/// Hex-encoded crypto fields for a `ConfidentialMPTConvertBack`.
struct ConfidentialMPTConvertBackBundle {
    holder_encrypted_amount: String,
    issuer_encrypted_amount: String,
    blinding_factor: String,
    balance_commitment: String,
    zk_proof: String,
}

/// Build the ConvertBack crypto: the revealed amount (encrypted under holder +
/// issuer keys), a Pedersen commitment to the current spending balance, and the
/// 816-byte proof linking that commitment to the on-ledger spending ciphertext
/// (read here) while range-proving `current_balance − amount ≥ 0`. The context
/// binds to the holder + issuance + sequence + on-ledger balance version.
async fn build_convert_back_material(
    setup: &ConfidentialSetup,
    sequence: u32,
    amount: u64,
    current_balance: u64,
) -> ConfidentialMPTConvertBackBundle {
    use xrpl::mpt_crypto::{commit, context, encrypt, prove, AccountId, Ciphertext, IssuanceId};

    let r = encrypt::random_blinding_factor().expect("blinding factor");
    let holder_ct = encrypt::encrypt(amount, &setup.holder_elgamal_pk, &r).expect("holder ct");
    let issuer_ct = encrypt::encrypt(amount, &setup.issuer_elgamal_pk, &r).expect("issuer ct");

    let balance_blinding = encrypt::random_blinding_factor().expect("balance blinding");
    let balance_commitment =
        commit::pedersen(current_balance, &balance_blinding).expect("balance commitment");

    let (cb_s, version) = onledger_spending(setup).await;
    let holder_account = account_id_bytes(&setup.holder.classic_address);
    let issuance = issuance_id_bytes(&setup.issuance_id);
    let ctx = context::convert_back(
        &AccountId::new(holder_account),
        &IssuanceId::new(issuance),
        sequence,
        version,
    )
    .expect("convert_back context hash");

    let proof = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey: &setup.holder_elgamal_sk,
        holder_pubkey: &setup.holder_elgamal_pk,
        amount,
        current_balance,
        context_hash: &ctx,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &Ciphertext::new(cb_s),
    })
    .expect("convert_back proof");

    ConfidentialMPTConvertBackBundle {
        holder_encrypted_amount: uppercase_hex(holder_ct.as_bytes()),
        issuer_encrypted_amount: uppercase_hex(issuer_ct.as_bytes()),
        blinding_factor: uppercase_hex(r.as_bytes()),
        balance_commitment: uppercase_hex(balance_commitment.as_bytes()),
        zk_proof: uppercase_hex(proof.as_bytes()),
    }
}

/// Set up a Send destination: a funded, authorized holder that has done a
/// Convert (so it has a registered `HolderEncryptionKey` and a confidential
/// inbox to receive into). Returns the wallet + its ElGamal keypair.
async fn setup_send_destination(
    setup: &ConfidentialSetup,
    client: &AsyncJsonRpcClient,
    seed_amount: u64,
) -> (Wallet, Privkey, Pubkey) {
    use xrpl::mpt_crypto::keypair;

    let dest = generate_funded_wallet().await;
    let (dest_sk, dest_pk) = keypair::generate().expect("destination ElGamal keypair");

    submit_signed(
        &dest.seed,
        serde_json::json!({
            "TransactionType": "MPTokenAuthorize",
            "Account": dest.classic_address,
            "MPTokenIssuanceID": setup.issuance_id,
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;
    submit_signed(
        &setup.issuer.seed,
        serde_json::json!({
            "TransactionType": "Payment",
            "Account": setup.issuer.classic_address,
            "Destination": dest.classic_address,
            "Amount": { "mpt_issuance_id": setup.issuance_id, "value": FUNDED_MPT },
            "Fee": MPT_TXN_FEE,
        }),
    )
    .await;
    // A Convert initializes the destination's confidential inbox + key.
    submit_first_convert(
        client,
        &dest,
        &dest_sk,
        &dest_pk,
        &setup.issuer_elgamal_pk,
        &setup.issuance_id,
        seed_amount,
    )
    .await;

    (dest, dest_sk, dest_pk)
}

/// `ConfidentialMPTSend`: a confidential holder-to-holder transfer. The sender
/// needs a spending balance (Convert + MergeInbox) and the destination an
/// initialized confidential inbox (its own Convert). SDK-built; expects
/// `tesSUCCESS` and verifies the sender is debited and the destination credited.
#[tokio::test]
async fn confidential_mpt_send() {
    with_blockchain_lock(|| async {
        // Send requires the issuance to allow transfers.
        let setup =
            setup_confidential_issuance(TF_MPT_CAN_CONFIDENTIAL_AMOUNT | TF_MPT_CAN_TRANSFER).await;
        let client = get_client().await;

        // Sender: convert + merge → spending balance.
        let sender_balance: u64 = 100;
        convert_public_to_confidential(&setup, client, sender_balance).await;
        merge_confidential_inbox(&setup).await;

        // Destination: authorized holder with an initialized confidential inbox.
        let dest_seed: u64 = 10;
        let (dest, dest_sk, dest_pk) = setup_send_destination(&setup, client, dest_seed).await;

        let amount: u64 = 40;
        let sequence =
            get_next_valid_seq_number(setup.holder.classic_address.clone().into(), client, None)
                .await
                .expect("fetch sender sequence");
        let m = build_send_material(&setup, &dest.classic_address, &dest_pk, sequence, amount, sender_balance)
            .await;

        let mut tx = ConfidentialMPTSend::new(
            setup.holder.classic_address.clone().into(),
            None,           // account_txn_id
            None,           // fee — autofilled (cMPT = 10× base)
            None,           // last_ledger_sequence
            None,           // memos
            Some(sequence), // bound into the proof context above
            None,           // signers
            None,           // source_tag
            None,           // ticket_sequence
            dest.classic_address.clone().into(),
            setup.issuance_id.clone().into(),
            m.sender_encrypted_amount.into(),
            m.destination_encrypted_amount.into(),
            m.issuer_encrypted_amount.into(),
            m.amount_commitment.into(),
            m.balance_commitment.into(),
            m.zk_proof.into(),
            None, // auditor_encrypted_amount (no auditor)
            None, // credential_ids
        );

        test_transaction(&mut tx, &setup.holder).await;

        // Sender spending debited; destination inbox credited.
        let sender_mpt = holder_mptoken(&setup.holder.classic_address, &setup.issuance_id).await;
        let sender_spending = decrypt_confidential_balance(
            sender_mpt["ConfidentialBalanceSpending"]
                .as_str()
                .expect("sender ConfidentialBalanceSpending present"),
            &setup.holder_elgamal_sk,
        );
        assert_eq!(
            sender_spending,
            sender_balance - amount,
            "sender spending should be debited by {amount}"
        );

        let dest_mpt = holder_mptoken(&dest.classic_address, &setup.issuance_id).await;
        let dest_inbox = decrypt_confidential_balance(
            dest_mpt["ConfidentialBalanceInbox"]
                .as_str()
                .expect("destination ConfidentialBalanceInbox present"),
            &dest_sk,
        );
        assert_eq!(
            dest_inbox,
            dest_seed + amount,
            "destination inbox should be credited by {amount}"
        );
    })
    .await;
}

/// Hex-encoded crypto fields for a `ConfidentialMPTSend`.
struct ConfidentialMPTSendBundle {
    sender_encrypted_amount: String,
    destination_encrypted_amount: String,
    issuer_encrypted_amount: String,
    amount_commitment: String,
    balance_commitment: String,
    zk_proof: String,
}

/// Build the Send crypto. CRITICAL invariant (XLS-0096 §5.4): one shared
/// `tx_blinding_factor` is the ElGamal randomness for *all three* participant
/// ciphertexts AND the Pedersen blinding for `amount_commitment`. The balance
/// commitment uses an independent blinding; the proof links it to the sender's
/// on-ledger spending ciphertext and range-proves `balance − amount ≥ 0`.
async fn build_send_material(
    setup: &ConfidentialSetup,
    destination_account: &str,
    destination_pk: &Pubkey,
    sequence: u32,
    amount: u64,
    current_balance: u64,
) -> ConfidentialMPTSendBundle {
    use xrpl::mpt_crypto::{commit, context, encrypt, prove, AccountId, Ciphertext, IssuanceId};

    // Shared randomness across all ciphertexts + the amount commitment.
    let tx_r = encrypt::random_blinding_factor().expect("tx blinding");
    let sender_ct = encrypt::encrypt(amount, &setup.holder_elgamal_pk, &tx_r).expect("sender ct");
    let dest_ct = encrypt::encrypt(amount, destination_pk, &tx_r).expect("dest ct");
    let issuer_ct = encrypt::encrypt(amount, &setup.issuer_elgamal_pk, &tx_r).expect("issuer ct");
    let amount_commitment = commit::pedersen(amount, &tx_r).expect("amount commitment");

    // Independent blinding for the balance commitment.
    let balance_blinding = encrypt::random_blinding_factor().expect("balance blinding");
    let balance_commitment =
        commit::pedersen(current_balance, &balance_blinding).expect("balance commitment");

    let (cb_s, version) = onledger_spending(setup).await;
    let sender_account = account_id_bytes(&setup.holder.classic_address);
    let dest_account = account_id_bytes(destination_account);
    let issuance = issuance_id_bytes(&setup.issuance_id);
    let ctx = context::send(
        &AccountId::new(sender_account),
        &IssuanceId::new(issuance),
        sequence,
        &AccountId::new(dest_account),
        version,
    )
    .expect("send context hash");

    let proof = prove::send(prove::SendProofParams {
        sender_privkey: &setup.holder_elgamal_sk,
        sender_pubkey: &setup.holder_elgamal_pk,
        amount,
        current_balance,
        tx_blinding_factor: &tx_r,
        context_hash: &ctx,
        amount_commitment: &amount_commitment,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &Ciphertext::new(cb_s),
        sender: prove::Participant {
            pubkey: &setup.holder_elgamal_pk,
            ciphertext: &sender_ct,
        },
        destination: prove::Participant {
            pubkey: destination_pk,
            ciphertext: &dest_ct,
        },
        issuer: prove::Participant {
            pubkey: &setup.issuer_elgamal_pk,
            ciphertext: &issuer_ct,
        },
        auditor: None,
    })
    .expect("send proof");

    ConfidentialMPTSendBundle {
        sender_encrypted_amount: uppercase_hex(sender_ct.as_bytes()),
        destination_encrypted_amount: uppercase_hex(dest_ct.as_bytes()),
        issuer_encrypted_amount: uppercase_hex(issuer_ct.as_bytes()),
        amount_commitment: uppercase_hex(amount_commitment.as_bytes()),
        balance_commitment: uppercase_hex(balance_commitment.as_bytes()),
        zk_proof: uppercase_hex(proof.as_bytes()),
    }
}
