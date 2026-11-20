// Scenarios (typed API, via submit_tx / test_transaction):
//   - all_or_nothing_success: single account, both inners applied
//   - all_or_nothing_rolls_back: one inner fails, nothing applied
//   - only_one / until_failure / independent: mode semantics, three accounts
//   - multi_account_swap: BatchSigners accepted, both sides applied
//   - fee: autofill matches (2 + n) x base + sum(inner); underpay rejected
// Scenarios (raw JSON, bypassing the client-side validator so the node's own
// checks are exercised, XLS-56 §2.3 / §3.3):
//   - outer flags, inner count, inner field rules, duplicates, sequence/ticket
//   - BatchSigners: missing/extra/outer/duplicate/unsorted/too many/bad sig/
//     replay under another outer account/wrong key
//   - tfInnerBatchTxn on a standalone transaction
//
// Not covered (need fixtures I can't see): multi-signed BatchSigners (Signers),
// disabled master key, pseudo-account signers, delegated inners, ticketed success.

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, submit_tx, test_transaction,
    with_blockchain_lock, SubmitOptions,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha512};
use xrpl::{
    asynch::{
        account::get_next_valid_seq_number,
        clients::{AsyncJsonRpcClient, XRPLAsyncClient},
        ledger::get_fee,
        transaction::calculate_fee_per_transaction_type,
    },
    core::{
        binarycodec::{
            encode, encode_for_signing, encode_for_signing_batch_v1_1, types::AccountId,
        },
        keypairs,
    },
    models::{
        requests::submit::Submit,
        results::submit::Submit as SubmitResult,
        transactions::{
            batch::{Batch, BatchFlag, RawTransaction},
            payment::Payment,
            CommonFields, CommonTransactionBuilder, TransactionType,
        },
        Amount, XRPAmount,
    },
    wallet::Wallet,
};

const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;
const ALL_OR_NOTHING: u32 = 0x0001_0000;
const ONLY_ONE: u32 = 0x0002_0000;
const NEW_ACCOUNT: &str = "20000000"; // 20 XRP: covers the standalone base reserve
const UNFUNDED: &str = "90000000000000000"; // valid amount, far above any balance -> tecUNFUNDED_PAYMENT

// ── helpers ──────────────────────────────────────────────────────────────────

async fn next_seq(c: &AsyncJsonRpcClient, address: &str) -> Option<u32> {
    get_next_valid_seq_number(address.to_string().into(), c, None)
        .await
        .ok()
}

/// A failed lookup is read as "account doesn't exist". Positive assertions
/// elsewhere in each test prove the node is reachable.
async fn exists(c: &AsyncJsonRpcClient, address: &str) -> bool {
    next_seq(c, address).await.is_some()
}

async fn base_fee(c: &AsyncJsonRpcClient) -> u64 {
    get_fee(c, None, None).await.unwrap().0.parse().unwrap()
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

fn raw(from: &str, to: &str, drops: &str, seq: u32) -> RawTransaction {
    RawTransaction::from_transaction(&payment_tx(from, to, drops, seq)).unwrap()
}

fn payment_inner(from: &str, to: &str, drops: &str, seq: u32) -> Value {
    json!({
        "TransactionType": "Payment", "Account": from, "Destination": to,
        "Amount": drops, "Sequence": seq, "Fee": "0",
        "Flags": TF_INNER_BATCH_TXN, "SigningPubKey": ""
    })
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
    let ids: Vec<String> = inners.iter().map(inner_id).collect();
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

/// Sign and submit arbitrary JSON, skipping every client-side check.
/// Returns the engine result, or `rpc_error: ..` when the node rejects the blob
/// outright (malformed, no result code).
async fn submit_json(c: &AsyncJsonRpcClient, mut tx: Value, signer: &Wallet) -> String {
    let payload = hex::decode(encode_for_signing(&tx).unwrap()).unwrap();
    tx["TxnSignature"] = json!(keypairs::sign(&payload, &signer.private_key).unwrap());
    let blob = encode(&tx).unwrap();
    let response = match c.request(Submit::new(None, blob.into(), None).into()).await {
        Ok(r) => r,
        Err(e) => return format!("rpc_error: {e:?}"),
    };
    let raw_response = format!("{response:?}"); // assumes XRPLResponse: Debug
    let parsed: Result<SubmitResult, _> = response.try_into();
    let result = match parsed {
        Ok(r) => r.engine_result.to_string(),
        Err(e) => format!("rpc_error: {e:?} | {raw_response}"),
    };
    ledger_accept().await;
    result
}

fn is_sig_rejection(result: &str) -> bool {
    result == "temINVALID" || result.starts_with("rpc_error")
}

// ── valid flows (typed API) ──────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_all_or_nothing_single_account() {
    with_blockchain_lock(|| async {
        let c = get_client().await;
        let w1 = generate_funded_wallet().await;
        let (r1, r2) = (Wallet::create(None).unwrap(), Wallet::create(None).unwrap());
        let s1 = next_seq(&c, &w1.classic_address).await.unwrap();

        // The outer consumes s1; inners from the same account follow it.
        let mut batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                raw(
                    &w1.classic_address,
                    &r1.classic_address,
                    NEW_ACCOUNT,
                    s1 + 1,
                ),
                raw(
                    &w1.classic_address,
                    &r2.classic_address,
                    NEW_ACCOUNT,
                    s1 + 2,
                ),
            ],
        )
        .with_sequence(s1);

        test_transaction(&mut batch, &w1).await;

        assert!(exists(&c, &r1.classic_address).await);
        assert!(exists(&c, &r2.classic_address).await);
        assert_eq!(next_seq(&c, &w1.classic_address).await, Some(s1 + 3));
    })
    .await;
}

struct Outcome {
    a_exists: bool,
    b_exists: bool,
    result: String,
    w1_before: u32,
    w1_after: u32,
}

/// Index 0 = w1's unfunded payment (fails), 1 = w2 -> new account A, 2 = w3 -> new account B.
/// Applied in `order`. The successful inners come from other accounts, so their
/// sequences don't depend on whether the failed inner consumes one.
async fn three_party(mode: BatchFlag, order: [usize; 3]) -> Outcome {
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

    let candidates = [
        raw(&w1.classic_address, &w2.classic_address, UNFUNDED, s1 + 1),
        raw(&w2.classic_address, &ra.classic_address, NEW_ACCOUNT, s2),
        raw(&w3.classic_address, &rb.classic_address, NEW_ACCOUNT, s3),
    ];
    let inners = order.iter().map(|&i| candidates[i].clone()).collect();
    let batch = Batch::new(w1.classic_address.clone(), mode, inners).with_sequence(s1);
    let signers = vec![
        batch
            .sign_as_batch_signer(&w2.classic_address, &w2)
            .unwrap(),
        batch
            .sign_as_batch_signer(&w3.classic_address, &w3)
            .unwrap(),
    ];
    let mut batch = batch.with_batch_signers(signers).unwrap();

    let result = submit_tx(
        &mut batch,
        SubmitOptions {
            wallet: &w1,
            autofill: true,
            check_fee: true,
        },
    )
    .await;
    ledger_accept().await;

    Outcome {
        a_exists: exists(&c, &ra.classic_address).await,
        b_exists: exists(&c, &rb.classic_address).await,
        result,
        w1_before: s1,
        w1_after: next_seq(&c, &w1.classic_address).await.unwrap(),
    }
}

#[tokio::test]
async fn test_batch_all_or_nothing_rolls_back() {
    with_blockchain_lock(|| async {
        let o = three_party(BatchFlag::TfAllOrNothing, [1, 0, 2]).await;
        assert_eq!(
            o.result, "tesSUCCESS",
            "outer succeeds even when inners fail (§2.5.1)"
        );
        assert!(!o.a_exists && !o.b_exists, "no inner may be applied");
        assert_eq!(
            o.w1_after,
            o.w1_before + 1,
            "only the outer consumes a sequence"
        );
    })
    .await;
}

#[tokio::test]
async fn test_batch_only_one() {
    with_blockchain_lock(|| async {
        let o = three_party(BatchFlag::TfOnlyOne, [0, 1, 2]).await;
        assert_eq!(o.result, "tesSUCCESS");
        assert!(o.a_exists, "first success is applied");
        assert!(!o.b_exists, "nothing after the first success");
    })
    .await;
}

#[tokio::test]
async fn test_batch_until_failure() {
    with_blockchain_lock(|| async {
        let o = three_party(BatchFlag::TfUntilFailure, [1, 0, 2]).await;
        assert_eq!(o.result, "tesSUCCESS");
        assert!(o.a_exists, "applied before the failure");
        assert!(!o.b_exists, "nothing after the first failure");
    })
    .await;
}

#[tokio::test]
async fn test_batch_independent() {
    with_blockchain_lock(|| async {
        let o = three_party(BatchFlag::TfIndependent, [0, 1, 2]).await;
        assert_eq!(o.result, "tesSUCCESS");
        assert!(o.a_exists && o.b_exists, "failures don't stop later inners");
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

        let batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                raw(&w1.classic_address, &w2.classic_address, "1000000", s1 + 1),
                raw(&w2.classic_address, &w1.classic_address, "2000000", s2),
            ],
        )
        .with_sequence(s1);
        let signer = batch
            .sign_as_batch_signer(&w2.classic_address, &w2)
            .unwrap();
        let mut batch = batch.with_batch_signers(vec![signer]).unwrap();

        test_transaction(&mut batch, &w1).await;

        assert_eq!(next_seq(&c, &w1.classic_address).await, Some(s1 + 2)); // outer + inner
        assert_eq!(next_seq(&c, &w2.classic_address).await, Some(s2 + 1)); // inner
    })
    .await;
}

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

        let batch = Batch::new(
            w1.classic_address.clone(),
            BatchFlag::TfAllOrNothing,
            vec![
                raw(&w1.classic_address, &w2.classic_address, "1000000", s1 + 1),
                raw(&w2.classic_address, &w1.classic_address, "1000000", s2),
            ],
        )
        .with_sequence(s1);
        let signer = batch
            .sign_as_batch_signer(&w2.classic_address, &w2)
            .unwrap();
        let batch = batch.with_batch_signers(vec![signer]).unwrap();

        // (2 + 1 BatchSigners signature) x base + 2 inner x base
        let expected = base * 5;
        let fee = calculate_fee_per_transaction_type(&batch, Some(c), None)
            .await
            .unwrap();
        assert_eq!(fee.0.parse::<u64>().unwrap(), expected);

        let mut tx = serde_json::to_value(&batch).unwrap();
        tx["SigningPubKey"] = json!(w1.public_key);
        tx["Fee"] = json!((expected - 1).to_string());
        assert_eq!(submit_json(&c, tx, &w1).await, "telINSUF_FEE_P");
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
        let seq = next_seq(&c, &w1.classic_address).await.unwrap();
        let base = base_fee(&c).await;
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

    /// Fee is generous so rejections come from the invariant under test, not the fee check.
    async fn submit(&self, flags: u32, inners: &[Value]) -> String {
        let fee = self.base * (6 + inners.len() as u64);
        let tx = batch_json(&self.w1, self.seq, flags, fee, inners, vec![]);
        submit_json(&self.c, tx, &self.w1).await
    }
}

async fn rejects_flags(flags: u32, expected: &str) {
    let s = Single::new().await;
    assert_eq!(s.submit(flags, &s.inners(2)).await, expected);
}

async fn rejects_inner(expected: &str, tamper: impl FnOnce(&mut Value, &Wallet)) {
    let s = Single::new().await;
    let mut inners = s.inners(2);
    tamper(&mut inners[0], &s.w1);
    assert_eq!(s.submit(ALL_OR_NOTHING, &inners).await, expected);
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
        assert_eq!(
            s.submit(ALL_OR_NOTHING, &s.inners(1)).await,
            "temARRAY_EMPTY"
        );
    })
    .await;
}

#[tokio::test]
async fn test_batch_rejects_more_than_eight_inners() {
    with_blockchain_lock(|| async {
        let s = Single::new().await;
        // Fails to deserialize: malformed, no result code (§2.3 #4)
        let r = s.submit(ALL_OR_NOTHING, &s.inners(9)).await;
        assert!(
            r.starts_with("rpc_error") || r.starts_with("tem"),
            "got {r}"
        );
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
        assert!(r.starts_with("rpc_error") || r.starts_with("tem"), "got {r}");
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
        let r = submit_json(&s.c, tx, &s.w1).await;
        assert!(
            r.starts_with("rpc_error") || r.starts_with("tem"),
            "got {r}"
        );
        assert_eq!(
            next_seq(&s.c, &s.w1.classic_address).await,
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
            s[i] = next_seq(&c, &wallet.classic_address).await.unwrap();
        }
        let base = base_fee(&c).await;
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
        let fee = self.base * (6 + inners.len() as u64 + signers.len() as u64);
        let tx = batch_json(&self.w[0], self.s[0], ALL_OR_NOTHING, fee, inners, signers);
        submit_json(&self.c, tx, &self.w[0]).await
    }
}

#[tokio::test]
async fn test_batch_signers_missing_all() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        assert_eq!(p.submit(&p.inners(), vec![]).await, "temBAD_SIGNER");
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
        let before = next_seq(&p.c, p.addr(0)).await.unwrap();
        let r = p.submit(&inners, sorted(entries)).await;
        println!("more_than_24 result: {r}");
        assert!(
            r == "temARRAY_TOO_LARGE" || r.starts_with("rpc_error"),
            "got {r}"
        );
        assert_eq!(
            next_seq(&p.c, p.addr(0)).await.unwrap(),
            before,
            "rejected batch must not consume the outer sequence"
        );
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
    })
    .await;
}

#[tokio::test]
async fn test_batch_signers_not_replayable_under_other_outer_account() {
    with_blockchain_lock(|| async {
        let p = Party::new().await;
        let other = generate_funded_wallet().await;
        let other_seq = next_seq(&p.c, &other.classic_address).await.unwrap();
        let inners = p.inners();
        // Required signers {w0, w1, w2}, all signed for outer = w0.
        let signers = sorted(vec![
            p.signer(0, &inners),
            p.signer(1, &inners),
            p.signer(2, &inners),
        ]);
        let fee = p.base * (6 + 3 + 3);
        let tx = batch_json(&other, other_seq, ALL_OR_NOTHING, fee, &inners, signers);
        let r = submit_json(&p.c, tx, &other).await;
        assert!(is_sig_rejection(&r), "got {r}");
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
    })
    .await;
}
