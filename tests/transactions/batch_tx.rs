// Scenarios, typed API (`test_transaction`), each with the node's view of the
// inner transactions checked by hash
// (hashSignedTx(inner) -> `tx` lookup -> meta.TransactionResult):
//   - all_or_nothing_single_account: both inners applied
//   - multi_account_swap: BatchSigners accepted, both sides applied
//   - fee: autofill matches (2 + n) x base + sum(inner); the exact fee is accepted
//   - mode matrix (typed path): all_or_nothing / only_one / until_failure / independent
// Scenarios, raw JSON (`submit_json`, bypassing the client-side validator so the
// node's own checks are exercised, XLS-56 §2.3 / §3.3):
//   - positive controls: a hand-built single-account batch and a fully signed
//     three-account batch are accepted and applied (so the negatives below can't
//     pass because of a bug in the raw-JSON helpers)
//   - mode matrix (raw path): same four modes as the typed path
//   - fee underpay rejected
//   - outer flags, inner count, inner field rules, duplicates, sequence/ticket
//   - BatchSigners: missing/extra/outer/duplicate/unsorted/too many/bad sig/
//     replay under another outer account/wrong key
//   - tfInnerBatchTxn on a standalone transaction
// A rejected batch must leave none of its inners in a ledger.

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha512};
use xrpl::{
    asynch::{
        clients::{AsyncJsonRpcClient, XRPLAsyncClient},
        ledger::get_fee,
        transaction::{autofill, calculate_fee_per_transaction_type},
    },
    core::{
        binarycodec::{
            encode, encode_for_signing, encode_for_signing_batch_v1_1, types::AccountId,
        },
        keypairs,
    },
    models::{
        requests::{submit::Submit, tx::Tx},
        results::submit::Submit as SubmitResult,
        transactions::{
            batch::{Batch, BatchFlag, RawTransaction},
            payment::Payment,
            CommonFields, CommonTransactionBuilder, TransactionType,
        },
        Amount, Model, XRPAmount,
    },
    signing::batch_signing::{combine_batch_signers, sign_multi_batch, MultisignMode},
    wallet::Wallet,
};

const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;
const ALL_OR_NOTHING: u32 = 0x0001_0000;
const ONLY_ONE: u32 = 0x0002_0000;
const NEW_ACCOUNT: &str = "20000000"; // 20 XRP: covers the standalone base reserve
const UNFUNDED: &str = "90000000000000000"; // valid amount, far above any balance -> tecUNFUNDED_PAYMENT

// ── node queries ─────────────────────────────────────────────────────────────

async fn next_seq(c: &AsyncJsonRpcClient, address: &str) -> Option<u32> {
    let response = c
        .request(
            xrpl::models::requests::account_info::AccountInfo::new(
                None,
                address.to_string().into(),
                None,
                None,
                None,
                None,
                Some(true),
            )
            .into(),
        )
        .await
        .unwrap_or_else(|e| panic!("account lookup transport failed for {address}: {e:?}"));
    let raw = response.raw_result.as_ref();
    let error = raw
        .and_then(|raw| raw.get("error"))
        .and_then(Value::as_str)
        .or(response.error.as_deref());
    if error == Some("actNotFound") {
        return None;
    }
    let raw =
        raw.unwrap_or_else(|| panic!("account lookup had no result for {address}: {response:?}"));
    Some(
        raw.get("account_data")
            .and_then(|data| data.get("Sequence"))
            .and_then(Value::as_u64)
            .and_then(|sequence| u32::try_from(sequence).ok())
            .unwrap_or_else(|| {
                panic!("account lookup returned an unexpected response for {address}: {raw}")
            }),
    )
}

/// Only the node's explicit `actNotFound` response is read as absence.
async fn exists(c: &AsyncJsonRpcClient, address: &str) -> bool {
    next_seq(c, address).await.is_some()
}

async fn base_fee(c: &AsyncJsonRpcClient) -> u64 {
    get_fee(c, None, None).await.unwrap().0.parse().unwrap()
}

/// `meta.TransactionResult` of the transaction with `hash`, or `None` when the
/// node has no such transaction (`txnNotFound`). Transport and response-shape
/// failures fail the test.
async fn tx_result(c: &AsyncJsonRpcClient, hash: &str) -> Option<String> {
    let response = c
        .request(Tx::new(None, None, None, None, Some(hash.to_string().into())).into())
        .await
        .unwrap_or_else(|e| panic!("tx lookup transport failed for {hash}: {e:?}"));

    let raw = response.raw_result.as_ref();
    let error = raw
        .and_then(|raw| raw.get("error"))
        .and_then(Value::as_str)
        .or(response.error.as_deref());
    if error == Some("txnNotFound") {
        return None;
    }
    let raw = raw.unwrap_or_else(|| panic!("tx lookup had no result for {hash}: {response:?}"));
    Some(
        raw.get("meta")
            .and_then(|meta| meta.get("TransactionResult"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("tx lookup for {hash} had no TransactionResult: {raw}"))
            .to_string(),
    )
}

/// The outer result is `tesSUCCESS` even when inners fail, so each inner is
/// looked up by its own hash and must be in a ledger with `tesSUCCESS`.
async fn verify_inner_transactions(c: &AsyncJsonRpcClient, hashes: &[String]) {
    for (i, hash) in hashes.iter().enumerate() {
        let result = tx_result(c, hash).await;
        assert_eq!(
            result.as_deref(),
            Some("tesSUCCESS"),
            "inner transaction {} ({hash}); None = not found in any ledger",
            i + 1
        );
    }
}

/// An unapplied inner is either absent from the ledger or carries a non-success result.
async fn assert_inner_not_applied(c: &AsyncJsonRpcClient, hash: &str) {
    let result = tx_result(c, hash).await;
    assert_ne!(
        result.as_deref(),
        Some("tesSUCCESS"),
        "inner transaction {hash} must not be applied"
    );
}

async fn assert_none_applied(c: &AsyncJsonRpcClient, inners: &[Value]) {
    for inner in inners {
        assert_inner_not_applied(c, &inner_id(inner)).await;
    }
}

// ── builders ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Path {
    /// `Batch` + `RawTransaction::from_transaction` + `test_transaction`.
    Typed,
    /// Hand-built JSON + `submit_json`.
    Raw,
}

fn payment_tx(from: &str, to: &str, drops: &str, seq: u32) -> Payment<'static> {
    Payment {
        common_fields: CommonFields {
            account: from.to_string().into(),
            transaction_type: TransactionType::Payment,
            ..Default::default()
        },
        amount: Amount::XRPAmount(XRPAmount::from(drops.to_string())),
        destination: to.to_string().into(),
        ..Default::default()
    }
    .with_sequence(seq)
}

fn payment_inner(from: &str, to: &str, drops: &str, seq: u32) -> Value {
    json!({
        "TransactionType": "Payment", "Account": from, "Destination": to,
        "Amount": drops, "Sequence": seq, "Fee": "0",
        "Flags": TF_INNER_BATCH_TXN, "SigningPubKey": ""
    })
}

/// One inner payment, buildable through either path.
struct Spec {
    from: String,
    to: String,
    drops: &'static str,
    seq: u32,
}

impl Spec {
    fn new(from: &str, to: &str, drops: &'static str, seq: u32) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            drops,
            seq,
        }
    }

    fn typed(&self) -> RawTransaction {
        RawTransaction::from_transaction(&payment_tx(&self.from, &self.to, self.drops, self.seq))
            .unwrap()
    }

    fn json(&self) -> Value {
        payment_inner(&self.from, &self.to, self.drops, self.seq)
    }

    fn hash(&self, path: Path) -> String {
        match path {
            Path::Typed => self.typed().transaction_hash().unwrap(),
            Path::Raw => inner_id(&self.json()),
        }
    }
}

fn inner_hashes(batch: &Batch<'_>) -> Vec<String> {
    batch
        .raw_transactions
        .iter()
        .map(|raw| raw.transaction_hash().unwrap())
        .collect()
}

fn json_hashes(inners: &[Value]) -> Vec<String> {
    inners.iter().map(inner_id).collect()
}

/// SHA-512Half(`TXN\0` || blob) of an inner object exactly as sent (no normalization).
fn inner_id(inner: &Value) -> String {
    let mut bytes = 0x5458_4E00u32.to_be_bytes().to_vec();
    bytes.extend(hex::decode(encode(inner).unwrap()).unwrap());
    hex::encode_upper(&Sha512::digest(&bytes)[..32])
}

fn account_id(address: &str) -> Vec<u8> {
    let id = AccountId::try_from(address).unwrap();
    let bytes: &[u8] = id.as_ref();
    bytes.to_vec()
}

/// `(account, {"BatchSigner": ...})` signed by `key` over the V1_1 payload.
fn signer_entry(
    outer: &str,
    outer_seq: u32,
    flags: u32,
    inners: &[Value],
    account: &str,
    key: &Wallet,
) -> (String, Value) {
    let ids = json_hashes(inners);
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let payload =
        encode_for_signing_batch_v1_1(outer, outer_seq, flags, &refs, account, None).unwrap();
    let signature = keypairs::sign(&hex::decode(payload).unwrap(), &key.private_key).unwrap();
    (
        account.to_string(),
        json!({"BatchSigner": {
            "Account": account, "SigningPubKey": key.public_key, "TxnSignature": signature
        }}),
    )
}

/// Ascending AccountID order, as the ledger requires.
fn sorted(mut entries: Vec<(String, Value)>) -> Vec<Value> {
    entries.sort_by_key(|(account, _)| account_id(account));
    entries.into_iter().map(|(_, v)| v).collect()
}

fn sign_and_combine<'a>(batch: &Batch<'a>, signers: &[&Wallet]) -> Batch<'a> {
    let fragments: Vec<_> = signers
        .iter()
        .map(|w| sign_multi_batch(w, batch, MultisignMode::Off, None).unwrap())
        .collect();

    combine_batch_signers(&fragments).unwrap()
}

fn batch_json(
    outer: &Wallet,
    seq: u32,
    flags: u32,
    fee: u64,
    inners: &[Value],
    signers: Vec<Value>,
) -> Value {
    let mut tx = json!({
        "TransactionType": "Batch", "Account": outer.classic_address, "Flags": flags,
        "Sequence": seq, "Fee": fee.to_string(), "SigningPubKey": outer.public_key,
        "RawTransactions": inners.iter().map(|i| json!({"RawTransaction": i})).collect::<Vec<_>>()
    });
    if !signers.is_empty() {
        tx["BatchSigners"] = Value::Array(signers);
    }
    tx
}

/// Generous on purpose, so a rejection comes from the invariant under test and
/// not from the fee check (exact fees are covered by the fee test).
fn generous_fee(base: u64, inners: usize, signers: usize) -> u64 {
    base * (6 + inners as u64 + signers as u64)
}

/// Sign and submit arbitrary JSON, skipping every client-side check.
/// Returns the node engine result; transport and response parsing failures fail the test.
async fn submit_json(c: &AsyncJsonRpcClient, mut tx: Value, signer: &Wallet) -> String {
    let payload = hex::decode(encode_for_signing(&tx).unwrap()).unwrap();
    tx["TxnSignature"] = json!(keypairs::sign(&payload, &signer.private_key).unwrap());
    let blob = encode(&tx).unwrap();
    let response = c
        .request(Submit::new(None, blob.into(), None).into())
        .await
        .unwrap_or_else(|e| panic!("submit transport failed: {e:?}"));

    let parsed: Result<SubmitResult, _> = response.clone().try_into();
    let result = match parsed {
        Ok(parsed) => parsed.engine_result.to_string(),
        Err(_) => {
            let raw = response.raw_result.unwrap_or(Value::Null);
            if raw.get("error").is_some() || response.error.is_some() {
                format!("node_rejection: {raw}")
            } else {
                panic!("submit response parsing failed without a node rejection: {raw}");
            }
        }
    };
    ledger_accept().await;
    result
}

fn is_sig_rejection(result: &str) -> bool {
    result == "temINVALID" || result.starts_with("node_rejection:")
}

fn assert_rejection(result: &str, expected: &str) {
    assert!(
        result == expected || result.starts_with("node_rejection:"),
        "expected {expected} or an explicit node rejection, got {result}"
    );
}

// ── client and raw JSON agree (no node) ──────────────────────────────────────

/// Ties the two paths together: the typed API and the hand-built JSON must hash
/// the same inners and produce byte-identical BatchSigners. If they drift, the
/// raw-JSON negatives below no longer test what the client sends.
#[test]
fn test_typed_and_raw_paths_agree() {
    assert_eq!(BatchFlag::TfAllOrNothing as u32, ALL_OR_NOTHING);
    assert_eq!(BatchFlag::TfOnlyOne as u32, ONLY_ONE);

    let (w1, w2) = (Wallet::create(None).unwrap(), Wallet::create(None).unwrap());
    let specs = [
        Spec::new(&w1.classic_address, &w2.classic_address, "1000000", 11),
        Spec::new(&w2.classic_address, &w1.classic_address, "2000000", 5),
    ];
    let batch = Batch::new(
        w1.classic_address.clone(),
        BatchFlag::TfAllOrNothing,
        specs.iter().map(Spec::typed).collect(),
        None,
    )
    .with_sequence(10);
    let raw_inners: Vec<Value> = specs.iter().map(Spec::json).collect();

    assert_eq!(inner_hashes(&batch), json_hashes(&raw_inners), "inner IDs");

    let fragment = sign_multi_batch(&w2, &batch, MultisignMode::Off, None).unwrap();
    let typed_entry = serde_json::to_value(&fragment.batch_signers.as_ref().unwrap()[0]).unwrap();
    let (_, raw_entry) = signer_entry(
        &w1.classic_address,
        10,
        ALL_OR_NOTHING,
        &raw_inners,
        &w2.classic_address,
        &w2,
    );
    assert_eq!(typed_entry, raw_entry, "BatchSigner");
}

// ── valid flows: typed API (test_transaction) ────────────────────────────────

#[tokio::test]
async fn test_batch_all_or_nothing_single_account() {
    with_blockchain_lock(|| async {
        let c = get_client().await;
        let w1 = generate_funded_wallet().await;
        let (r1, r2) = (Wallet::create(None).unwrap(), Wallet::create(None).unwrap());
        let s1 = next_seq(c, &w1.classic_address).await.unwrap();

        // The outer consumes s1; inners from the same account follow it.
        let mut batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                Spec::new(
                    &w1.classic_address,
                    &r1.classic_address,
                    NEW_ACCOUNT,
                    s1 + 1,
                )
                .typed(),
                Spec::new(
                    &w1.classic_address,
                    &r2.classic_address,
                    NEW_ACCOUNT,
                    s1 + 2,
                )
                .typed(),
            ],
            None,
        )
        .with_sequence(s1);
        let hashes = inner_hashes(&batch);

        test_transaction(&mut batch, &w1).await;

        verify_inner_transactions(c, &hashes).await;
        assert!(exists(c, &r1.classic_address).await);
        assert!(exists(c, &r2.classic_address).await);
        assert_eq!(next_seq(c, &w1.classic_address).await, Some(s1 + 3));
    })
    .await;
}

#[tokio::test]
async fn test_batch_multi_account_swap() {
    with_blockchain_lock(|| async {
        let c = get_client().await;
        let (w1, w2) = (
            generate_funded_wallet().await,
            generate_funded_wallet().await,
        );
        let s1 = next_seq(c, &w1.classic_address).await.unwrap();
        let s2 = next_seq(c, &w2.classic_address).await.unwrap();

        let mut batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                Spec::new(&w1.classic_address, &w2.classic_address, "1000000", s1 + 1).typed(),
                Spec::new(&w2.classic_address, &w1.classic_address, "2000000", s2).typed(),
            ],
            None,
        )
        .with_sequence(s1);

        autofill(&mut batch, c, Some(2)).await.unwrap();

        let mut combined = sign_and_combine(&batch, &[&w2]);
        let hashes = inner_hashes(&combined);

        test_transaction(&mut combined, &w1).await;

        verify_inner_transactions(c, &hashes).await;
        assert_eq!(next_seq(c, &w1.classic_address).await, Some(s1 + 2)); // outer + inner
        assert_eq!(next_seq(c, &w2.classic_address).await, Some(s2 + 1)); // inner
    })
    .await;
}

// ── valid flows: raw JSON (submit_json), positive controls ───────────────────

#[tokio::test]
async fn test_batch_single_account_raw_json_accepted() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let inners = s.inners(2);

        assert_eq!(s.submit(ALL_OR_NOTHING, &inners).await, "tesSUCCESS");

        verify_inner_transactions(s.c, &json_hashes(&inners)).await;
        assert_eq!(next_seq(s.c, &s.w1.classic_address).await, Some(s.seq + 3));
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_complete_set_accepted() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let signers = sorted(vec![p.signer(1, &inners), p.signer(2, &inners)]);

        assert_eq!(p.submit(&inners, signers).await, "tesSUCCESS");

        verify_inner_transactions(p.c, &json_hashes(&inners)).await;
        assert_eq!(next_seq(p.c, p.addr(0)).await, Some(p.s[0] + 2)); // outer + inner
        assert_eq!(next_seq(p.c, p.addr(1)).await, Some(p.s[1] + 1));
        assert_eq!(next_seq(p.c, p.addr(2)).await, Some(p.s[2] + 1));
    })
    .await;
}

// ── mode matrix: same semantics through both paths ───────────────────────────

struct Outcome {
    /// Engine result of the outer transaction. Raw path only: `test_transaction`
    /// already requires the outer to succeed.
    result: Option<String>,
    /// Whether each candidate (0, 1, 2 below) is in a ledger with `tesSUCCESS`, by hash.
    applied: [bool; 3],
    a_exists: bool,
    b_exists: bool,
    w1_before: u32,
    w1_after: u32,
}

/// Candidate 0 = w1's unfunded payment (fails), 1 = w2 -> new account A,
/// 2 = w3 -> new account B, applied in `order`. The successful inners come from
/// other accounts, so their sequences don't depend on whether the failed inner
/// consumes one.
async fn three_party(mode: BatchFlag, order: [usize; 3], path: Path) -> Outcome {
    let c = get_client().await;
    let (w1, w2, w3) = (
        generate_funded_wallet().await,
        generate_funded_wallet().await,
        generate_funded_wallet().await,
    );
    let (ra, rb) = (Wallet::create(None).unwrap(), Wallet::create(None).unwrap());
    let s1 = next_seq(c, &w1.classic_address).await.unwrap();
    let s2 = next_seq(c, &w2.classic_address).await.unwrap();
    let s3 = next_seq(c, &w3.classic_address).await.unwrap();

    let specs = [
        Spec::new(&w1.classic_address, &w2.classic_address, UNFUNDED, s1 + 1),
        Spec::new(&w2.classic_address, &ra.classic_address, NEW_ACCOUNT, s2),
        Spec::new(&w3.classic_address, &rb.classic_address, NEW_ACCOUNT, s3),
    ];
    let hashes: Vec<String> = specs.iter().map(|s| s.hash(path)).collect();
    let ordered: Vec<&Spec> = order.iter().map(|&i| &specs[i]).collect();

    let result = match path {
        Path::Typed => {
            let mut batch = Batch::new(
                w1.classic_address.clone(),
                mode,
                ordered.iter().map(|s| s.typed()).collect(),
                None,
            )
            .with_sequence(s1);
            autofill(&mut batch, c, Some(2)).await.unwrap();

            let mut combined = sign_and_combine(&batch, &[&w2, &w3]);
            test_transaction(&mut combined, &w1).await;
            None
        }
        Path::Raw => {
            let flags = mode as u32;
            let inners: Vec<Value> = ordered.iter().map(|s| s.json()).collect();
            let signers = sorted(vec![
                signer_entry(
                    &w1.classic_address,
                    s1,
                    flags,
                    &inners,
                    &w2.classic_address,
                    &w2,
                ),
                signer_entry(
                    &w1.classic_address,
                    s1,
                    flags,
                    &inners,
                    &w3.classic_address,
                    &w3,
                ),
            ]);
            let fee = generous_fee(base_fee(c).await, inners.len(), signers.len());
            let tx = batch_json(&w1, s1, flags, fee, &inners, signers);
            Some(submit_json(c, tx, &w1).await)
        }
    };

    let mut applied = [false; 3];
    for (i, hash) in hashes.iter().enumerate() {
        applied[i] = tx_result(c, hash).await.as_deref() == Some("tesSUCCESS");
    }

    Outcome {
        result,
        applied,
        a_exists: exists(c, &ra.classic_address).await,
        b_exists: exists(c, &rb.classic_address).await,
        w1_before: s1,
        w1_after: next_seq(c, &w1.classic_address).await.unwrap(),
    }
}

/// `a` / `b`: whether inner A (w2 -> new account) and inner B (w3 -> new account)
/// are applied. Checked both by hash and by ledger state.
fn assert_outcome(o: &Outcome, a: bool, b: bool) {
    if let Some(result) = &o.result {
        assert_eq!(
            result, "tesSUCCESS",
            "outer succeeds even when inners fail (§2.5.1)"
        );
    }
    assert!(!o.applied[0], "w1's unfunded payment never succeeds");
    assert_eq!(o.applied[1], a, "inner A by hash");
    assert_eq!(o.applied[2], b, "inner B by hash");
    assert_eq!(o.a_exists, a, "account A in the ledger");
    assert_eq!(o.b_exists, b, "account B in the ledger");
}

async fn check_all_or_nothing_rolls_back(path: Path) {
    let o = three_party(BatchFlag::TfAllOrNothing, [1, 0, 2], path).await;
    assert_outcome(&o, false, false); // no inner may be applied
    assert_eq!(
        o.w1_after,
        o.w1_before + 1,
        "only the outer consumes a sequence"
    );
}

async fn check_only_one(path: Path) {
    let o = three_party(BatchFlag::TfOnlyOne, [0, 1, 2], path).await;
    assert_outcome(&o, true, false); // first success applied, nothing after it
}

async fn check_until_failure(path: Path) {
    let o = three_party(BatchFlag::TfUntilFailure, [1, 0, 2], path).await;
    assert_outcome(&o, true, false); // applied before the failure, nothing after it
}

async fn check_independent(path: Path) {
    let o = three_party(BatchFlag::TfIndependent, [0, 1, 2], path).await;
    assert_outcome(&o, true, true); // failures don't stop later inners
}

macro_rules! mode_tests {
    ($($name:ident => $check:ident($path:expr)),* $(,)?) => {
        $(
            #[tokio::test]
            async fn $name() {
                with_blockchain_lock(|| async { $check($path).await }).await;
            }
        )*
    };
}

mode_tests! {
    test_batch_all_or_nothing_rolls_back_typed => check_all_or_nothing_rolls_back(Path::Typed),
    test_batch_all_or_nothing_rolls_back_raw => check_all_or_nothing_rolls_back(Path::Raw),
    test_batch_only_one_typed => check_only_one(Path::Typed),
    test_batch_only_one_raw => check_only_one(Path::Raw),
    test_batch_until_failure_typed => check_until_failure(Path::Typed),
    test_batch_until_failure_raw => check_until_failure(Path::Raw),
    test_batch_independent_typed => check_independent(Path::Typed),
    test_batch_independent_raw => check_independent(Path::Raw),
}

// ── fee ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_fee_matches_spec_and_underpay_rejected() {
    with_blockchain_lock(|| async {
        let c = get_client().await;
        let base = base_fee(c).await;

        let (w1, w2) = (
            generate_funded_wallet().await,
            generate_funded_wallet().await,
        );

        let s1 = next_seq(c, &w1.classic_address).await.unwrap();
        let s2 = next_seq(c, &w2.classic_address).await.unwrap();

        let mut batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                Spec::new(&w1.classic_address, &w2.classic_address, "1000000", s1 + 1).typed(),
                Spec::new(&w2.classic_address, &w1.classic_address, "1000000", s2).typed(),
            ],
            None,
        )
        .with_sequence(s1);

        autofill(&mut batch, c, Some(1)).await.unwrap();

        let mut combined = sign_and_combine(&batch, &[&w2]);
        let hashes = inner_hashes(&combined);

        // (2 + 1 BatchSigners signature) x base + 2 inner x base
        let expected = base * 5;
        let fee = calculate_fee_per_transaction_type(&combined, Some(c), None)
            .await
            .unwrap();

        assert_eq!(fee.0.parse::<u64>().unwrap(), expected);

        // One drop short is refused by the node. The batch is fully signed, so the
        // fee is the only defect; raw JSON bypasses the client.
        let mut tx = serde_json::to_value(&combined).unwrap();
        tx["SigningPubKey"] = json!(w1.public_key);
        tx["Fee"] = json!((expected - 1).to_string());
        assert_eq!(submit_json(c, tx, &w1).await, "telINSUF_FEE_P");
        for hash in &hashes {
            assert_inner_not_applied(c, hash).await;
        }

        // The exact fee is accepted (typed API).
        test_transaction(&mut combined, &w1).await;

        verify_inner_transactions(c, &hashes).await;
    })
    .await;
}

// ── node-side invariants: single account ─────────────────────────────────────

struct Single<'a> {
    c: &'a AsyncJsonRpcClient,
    w1: Wallet,
    dest: Wallet,
    seq: u32,
    base: u64,
}

impl<'a> Single<'a> {
    async fn new() -> Self {
        let c = get_client().await;
        let (w1, dest) = (
            generate_funded_wallet().await,
            generate_funded_wallet().await,
        );
        let seq = next_seq(c, &w1.classic_address).await.unwrap();
        let base = base_fee(c).await;
        Self {
            c,
            w1,
            dest,
            seq,
            base,
        }
    }

    fn inners(&self, n: u32) -> Vec<Value> {
        (1..=n)
            .map(|i| {
                payment_inner(
                    &self.w1.classic_address,
                    &self.dest.classic_address,
                    "1000000",
                    self.seq + i,
                )
            })
            .collect()
    }

    async fn submit(&self, flags: u32, inners: &[Value]) -> String {
        let fee = generous_fee(self.base, inners.len(), 0);
        let tx = batch_json(&self.w1, self.seq, flags, fee, inners, vec![]);
        submit_json(self.c, tx, &self.w1).await
    }
}

async fn rejects_flags(flags: u32, expected: &str) {
    let s = Single::new().await;
    let inners = s.inners(2);
    assert_rejection(&s.submit(flags, &inners).await, expected);
    assert_none_applied(s.c, &inners).await;
}

async fn rejects_inner(expected: &str, tamper: impl FnOnce(&mut Value, &Wallet)) {
    let s = Single::new().await;
    let mut inners = s.inners(2);
    tamper(&mut inners[0], &s.w1);
    assert_eq!(s.submit(ALL_OR_NOTHING, &inners).await, expected);
    assert_none_applied(s.c, &inners).await;
}

#[tokio::test]
async fn test_batch_rejects_no_mode_flag() {
    with_blockchain_lock(|| async { rejects_flags(0, "temINVALID_FLAG").await }).await;
}

#[tokio::test]
async fn test_batch_rejects_two_mode_flags() {
    with_blockchain_lock(|| async {
        rejects_flags(ALL_OR_NOTHING | ONLY_ONE, "temINVALID_FLAG").await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_fewer_than_two_inners() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let inners = s.inners(1);
        assert_eq!(s.submit(ALL_OR_NOTHING, &inners).await, "temARRAY_EMPTY");
        assert_none_applied(s.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_more_than_eight_inners() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let inners = s.inners(9);
        // Fails to deserialize: malformed, no result code (§2.3 #4)
        let r = s.submit(ALL_OR_NOTHING, &inners).await;
        assert!(
            r.starts_with("node_rejection:") || r.starts_with("tem"),
            "got {r}"
        );
        assert_none_applied(s.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_nested_batch() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let mut inners = s.inners(2);
        inners[0] = json!({
            "TransactionType": "Batch", "Account": s.w1.classic_address,
            "Flags": TF_INNER_BATCH_TXN | ALL_OR_NOTHING, "Sequence": s.seq + 1,
            "Fee": "0", "SigningPubKey": "",
            "RawTransactions": s.inners(2).iter().map(|i| json!({"RawTransaction": i})).collect::<Vec<_>>()
        });
        // Malformed before preflight (§2.3 #10)
        let r = s.submit(ALL_OR_NOTHING, &inners).await;
        assert!(r.starts_with("node_rejection:") || r.starts_with("tem"), "got {r}");
        assert_none_applied(s.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_nonzero_fee() {
    with_blockchain_lock(|| async {
        rejects_inner("temBAD_FEE", |i, _| i["Fee"] = json!("10")).await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_signature() {
    with_blockchain_lock(|| async {
        rejects_inner("temBAD_SIGNATURE", |i, _| i["TxnSignature"] = json!("ABCD")).await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_signing_pub_key() {
    with_blockchain_lock(|| async {
        rejects_inner("temBAD_REGKEY", |i, w| {
            i["SigningPubKey"] = json!(w.public_key)
        })
        .await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_signers() {
    with_blockchain_lock(|| async {
        rejects_inner("temBAD_SIGNER", |i, w| {
            i["Signers"] = json!([{"Signer": {
                "Account": w.classic_address, "SigningPubKey": w.public_key, "TxnSignature": "ABCD"
            }}]);
        })
        .await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_without_inner_flag() {
    with_blockchain_lock(|| async {
        rejects_inner("temINVALID_FLAG", |i, _| i["Flags"] = json!(0)).await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_inner_failing_preflight() {
    with_blockchain_lock(|| async {
        // XRP self-payment is invalid on its own -> temINVALID_INNER_BATCH (§2.3 #14)
        rejects_inner("temINVALID_INNER_BATCH", |i, w| {
            i["Destination"] = json!(w.classic_address)
        })
        .await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_duplicate_inner() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let mut inners = s.inners(2);
        inners[1] = inners[0].clone();
        assert_eq!(s.submit(ALL_OR_NOTHING, &inners).await, "temREDUNDANT");
        assert_none_applied(s.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_sequence_zero_without_ticket() {
    with_blockchain_lock(|| async {
        rejects_inner("temSEQ_AND_TICKET", |i, _| i["Sequence"] = json!(0)).await
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_sequence_with_ticket() {
    with_blockchain_lock(|| async {
        rejects_inner("temSEQ_AND_TICKET", |i, _| i["TicketSequence"] = json!(5)).await
    })
    .await;
}

#[tokio::test]
async fn test_inner_flag_rejected_outside_batch() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        let tx = json!({
            "TransactionType": "Payment", "Account": s.w1.classic_address,
            "Destination": s.dest.classic_address, "Amount": "1000000",
            "Sequence": s.seq, "Fee": s.base.to_string(),
            "Flags": TF_INNER_BATCH_TXN, "SigningPubKey": s.w1.public_key
        });
        // §5.2: the submit RPC refuses inner transactions outright, so an RPC-level
        // error is expected; an engine result is also acceptable.
        let r = submit_json(s.c, tx, &s.w1).await;
        assert!(
            r.starts_with("node_rejection:") || r.starts_with("tem"),
            "got {r}"
        );
        assert_eq!(
            next_seq(s.c, &s.w1.classic_address).await,
            Some(s.seq),
            "an unsigned inner transaction must never be applied"
        );
    })
    .await;
}

// ── node-side invariants: BatchSigners ───────────────────────────────────────

/// w[0] is the outer account. Inners come from all three; w[1] and w[2] must sign.
struct Party<'a> {
    c: &'a AsyncJsonRpcClient,
    w: [Wallet; 3],
    s: [u32; 3],
    base: u64,
}

impl<'a> Party<'a> {
    async fn new() -> Self {
        let c = get_client().await;
        let w = [
            generate_funded_wallet().await,
            generate_funded_wallet().await,
            generate_funded_wallet().await,
        ];
        let mut s = [0u32; 3];
        for (i, wallet) in w.iter().enumerate() {
            s[i] = next_seq(c, &wallet.classic_address).await.unwrap();
        }
        let base = base_fee(c).await;
        Self { c, w, s, base }
    }

    fn addr(&self, i: usize) -> &str {
        &self.w[i].classic_address
    }

    fn inners(&self) -> Vec<Value> {
        vec![
            payment_inner(self.addr(0), self.addr(2), "1000000", self.s[0] + 1),
            payment_inner(self.addr(1), self.addr(2), "1000000", self.s[1]),
            payment_inner(self.addr(2), self.addr(0), "1000000", self.s[2]),
        ]
    }

    /// Entry for `who`, signed over the payload bound to the real outer account/sequence.
    fn signer(&self, who: usize, inners: &[Value]) -> (String, Value) {
        signer_entry(
            self.addr(0),
            self.s[0],
            ALL_OR_NOTHING,
            inners,
            self.addr(who),
            &self.w[who],
        )
    }

    async fn submit(&self, inners: &[Value], signers: Vec<Value>) -> String {
        let fee = generous_fee(self.base, inners.len(), signers.len());
        let tx = batch_json(&self.w[0], self.s[0], ALL_OR_NOTHING, fee, inners, signers);
        submit_json(self.c, tx, &self.w[0]).await
    }
}

#[tokio::test]
async fn test_batch_signers_missing_all() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        assert_eq!(p.submit(&inners, vec![]).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_missing_one() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let signers = sorted(vec![p.signer(1, &inners)]);
        assert_eq!(p.submit(&inners, signers).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_extra_signer() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let stranger = Wallet::create(None).unwrap();
        let extra = signer_entry(
            p.addr(0),
            p.s[0],
            ALL_OR_NOTHING,
            &inners,
            &stranger.classic_address,
            &stranger,
        );
        let signers = sorted(vec![p.signer(1, &inners), p.signer(2, &inners), extra]);
        assert_eq!(p.submit(&inners, signers).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_contains_outer_account() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let signers = sorted(vec![
            p.signer(0, &inners),
            p.signer(1, &inners),
            p.signer(2, &inners),
        ]);
        assert_eq!(p.submit(&inners, signers).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_duplicate() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let mut signers = sorted(vec![p.signer(1, &inners), p.signer(2, &inners)]);
        signers.insert(0, signers[0].clone());
        assert_eq!(p.submit(&inners, signers).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_not_sorted() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let mut signers = sorted(vec![p.signer(1, &inners), p.signer(2, &inners)]);
        signers.reverse();
        assert_eq!(p.submit(&inners, signers).await, "temBAD_SIGNER");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_more_than_24() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let mut entries = vec![p.signer(1, &inners), p.signer(2, &inners)];
        for _ in 0..23 {
            let extra = Wallet::create(None).unwrap();
            entries.push(signer_entry(
                p.addr(0),
                p.s[0],
                ALL_OR_NOTHING,
                &inners,
                &extra.classic_address,
                &extra,
            ));
        }
        let before = next_seq(p.c, p.addr(0)).await.unwrap();
        let r = p.submit(&inners, sorted(entries)).await;
        println!("more_than_24 result: {r}");
        assert!(
            r == "temARRAY_TOO_LARGE" || r.starts_with("node_rejection:"),
            "got {r}"
        );
        assert_eq!(
            next_seq(p.c, p.addr(0)).await.unwrap(),
            before,
            "rejected batch must not consume the outer sequence"
        );
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_bad_signature() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        // Signed for outer sequence + 1, so the payload doesn't match what is submitted.
        let stale = signer_entry(
            p.addr(0),
            p.s[0] + 1,
            ALL_OR_NOTHING,
            &inners,
            p.addr(1),
            &p.w[1],
        );
        let signers = sorted(vec![stale, p.signer(2, &inners)]);
        let r = p.submit(&inners, signers).await;
        assert!(is_sig_rejection(&r), "got {r}");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_not_replayable_under_other_outer_account() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let other = generate_funded_wallet().await;
        let other_seq = next_seq(p.c, &other.classic_address).await.unwrap();
        let inners = p.inners();
        // Required signers {w0, w1, w2}, all signed for outer = w0.
        let fee = generous_fee(p.base, inners.len(), 3);
        let signers = sorted(
            (0..3)
                .map(|i| {
                    signer_entry(
                        p.addr(0),
                        other_seq,
                        ALL_OR_NOTHING,
                        &inners,
                        p.addr(i),
                        &p.w[i],
                    )
                })
                .collect(),
        );
        let tx = batch_json(&other, other_seq, ALL_OR_NOTHING, fee, &inners, signers);
        let r = submit_json(p.c, tx, &other).await;
        assert!(is_sig_rejection(&r), "got {r}");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_wrong_key_for_account() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let inners = p.inners();
        let stranger = Wallet::create(None).unwrap();
        // Valid signature, but not by a key authorized for w1's account.
        let forged = signer_entry(
            p.addr(0),
            p.s[0],
            ALL_OR_NOTHING,
            &inners,
            p.addr(1),
            &stranger,
        );
        let signers = sorted(vec![forged, p.signer(2, &inners)]);
        assert_eq!(p.submit(&inners, signers).await, "tefBAD_AUTH");
        assert_none_applied(p.c, &inners).await;
    })
    .await;
}

// ── client-side opt-in check agrees with the node ────────────────────────────

#[tokio::test]
async fn test_batch_validate_signers_agrees_with_node() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let batch = Batch::new(
            p.addr(0).to_string(),
            BatchFlag::TfAllOrNothing,
            vec![
                Spec::new(p.addr(0), p.addr(2), "1000000", p.s[0] + 1).typed(),
                Spec::new(p.addr(1), p.addr(2), "1000000", p.s[1]).typed(),
                Spec::new(p.addr(2), p.addr(0), "1000000", p.s[2]).typed(),
            ],
            None,
        )
        .with_sequence(p.s[0]);
        let hashes = inner_hashes(&batch);

        // Partial: valid draft, fails the opt-in check, rejected by the node (raw JSON).
        let partial = sign_and_combine(&batch, &[&p.w[1]]);
        assert!(partial.get_errors().is_ok());
        assert!(partial.validate_batch_signers().is_err());
        let mut tx = serde_json::to_value(&partial).unwrap();
        tx["SigningPubKey"] = json!(p.w[0].public_key);
        tx["Fee"] = json!(generous_fee(p.base, 3, 1).to_string());
        assert_eq!(submit_json(p.c, tx, &p.w[0]).await, "temBAD_SIGNER");
        for hash in &hashes {
            assert_inner_not_applied(p.c, hash).await;
        }

        // Complete: passes the opt-in check, accepted and applied (typed API).
        let mut full = sign_and_combine(&batch, &[&p.w[1], &p.w[2]]);
        assert!(full.validate_batch_signers().is_ok());
        test_transaction(&mut full, &p.w[0]).await;
        verify_inner_transactions(p.c, &hashes).await;
    })
    .await;
}
