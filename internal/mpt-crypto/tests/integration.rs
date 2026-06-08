//! Integration tests for the safe wrapper.
//!
//! Each test exercises one slice of the API end-to-end. Where the safe
//! wrapper produces a proof, we check it via the corresponding
//! [`mpt_crypto::verify`] function — which calls the same C verifier rippled
//! uses, so it is the oracle for "did our prove call produce something the
//! ledger would accept?". These tests use only the safe surface: no `unsafe`
//! and no direct `mpt_crypto_sys` calls.

use mpt_crypto::*;

// ─────────────────────────────────────────────────────────────────────────
//  Encryption round-trip
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn keypair_encrypt_decrypt_roundtrip() {
    let (sk, pk) = keypair::generate().expect("keypair");
    let r = encrypt::random_blinding_factor().expect("blinding");
    let ct = encrypt::encrypt(1234, &pk, &r).expect("encrypt");
    let m = encrypt::decrypt(&ct, &sk).expect("decrypt");
    assert_eq!(m, 1234);
}

#[test]
fn encrypt_with_same_blinding_is_deterministic() {
    let (_sk, pk) = keypair::generate().unwrap();
    let r = encrypt::random_blinding_factor().unwrap();

    let ct1 = encrypt::encrypt(42, &pk, &r).unwrap();
    let ct2 = encrypt::encrypt(42, &pk, &r).unwrap();
    assert_eq!(
        ct1.as_bytes(),
        ct2.as_bytes(),
        "same (m, r, pk) should produce identical ciphertext"
    );
}

#[test]
fn pubkey_debug_is_truncated_for_readability() {
    // `Pubkey` is public information — its `Debug` impl truncates for
    // readability (first 4 + last 4 bytes), not for security. Contrast with
    // `privkey_debug_does_not_leak_bytes` below, which checks actual
    // redaction of secret material.
    let (_sk, pk) = keypair::generate().unwrap();
    let s = format!("{pk:?}");
    assert!(s.starts_with("Pubkey("));
    assert!(s.len() < 30, "Debug output unexpectedly long: {s}");
}

#[test]
fn privkey_debug_does_not_leak_bytes() {
    let (sk, _pk) = keypair::generate().unwrap();
    let s = format!("{sk:?}");
    assert!(
        s.contains("redacted"),
        "Privkey Debug must not expose bytes"
    );
    // The hex of any bit of the actual key should not appear in the Debug output.
    let hex: String = sk.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    assert!(
        !s.contains(&hex[0..8]),
        "Privkey Debug output appears to contain key bytes"
    );
}

// ─────────────────────────────────────────────────────────────────────────
//  Pedersen commitment determinism
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn pedersen_commitment_is_deterministic() {
    let r = encrypt::random_blinding_factor().unwrap();
    let c1 = commit::pedersen(7, &r).unwrap();
    let c2 = commit::pedersen(7, &r).unwrap();
    assert_eq!(c1, c2, "same (amount, blinding) should commit identically");
}

#[test]
fn pedersen_commitment_changes_with_amount() {
    let r = encrypt::random_blinding_factor().unwrap();
    let c1 = commit::pedersen(7, &r).unwrap();
    let c2 = commit::pedersen(8, &r).unwrap();
    assert_ne!(c1, c2);
}

/// `convert_back_remainder` computes `pc_rem = pc_b - m·G`. Since
/// `pc_b = balance·G + ρ·H`, the remainder must equal a fresh commitment to
/// `(balance - m)` under the same blinding `ρ`. This pins the helper to its
/// algebraic definition without needing a proof.
#[test]
fn convert_back_remainder_matches_direct_commitment() {
    let balance = 1_000u64;
    let withdraw = 250u64;
    let rho = encrypt::random_blinding_factor().unwrap();

    let pc_b = commit::pedersen(balance, &rho).unwrap();
    let remainder = commit::convert_back_remainder(&pc_b, withdraw).unwrap();
    let direct = commit::pedersen(balance - withdraw, &rho).unwrap();

    assert_eq!(
        remainder, direct,
        "remainder commitment did not match a direct commitment to (balance - amount)"
    );
}

// ─────────────────────────────────────────────────────────────────────────
//  Context hash variants are distinct
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn context_hashes_differ_per_transaction_type() {
    let acc = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let dst = AccountId::new([3; 20]);
    let seq = 100;
    let ver = 5;

    let h_convert = context::convert(&acc, &iss, seq).unwrap();
    let h_convert_back = context::convert_back(&acc, &iss, seq, ver).unwrap();
    let h_send = context::send(&acc, &iss, seq, &dst, ver).unwrap();
    let h_clawback = context::clawback(&acc, &iss, seq, &dst).unwrap();

    let all = [h_convert, h_convert_back, h_send, h_clawback];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(
                all[i], all[j],
                "context hashes for different tx types collided ({i} vs {j})"
            );
        }
    }
}

#[test]
fn send_context_hash_binds_to_destination() {
    let snd = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let d1 = AccountId::new([3; 20]);
    let d2 = AccountId::new([4; 20]);
    let h1 = context::send(&snd, &iss, 1, &d1, 0).unwrap();
    let h2 = context::send(&snd, &iss, 1, &d2, 0).unwrap();
    assert_ne!(h1, h2, "Send proof must be unforgeable across destinations");
}

// ─────────────────────────────────────────────────────────────────────────
//  Convert proof — Schnorr PoK round-trip via the safe verifier
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn convert_proof_verifies() {
    let (sk, pk) = keypair::generate().unwrap();
    let acc = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let ctx = context::convert(&acc, &iss, 1).unwrap();

    let proof = prove::convert(&sk, &pk, &ctx).unwrap();

    verify::convert(&proof, &pk, &ctx).expect("convert proof failed verification");
}

#[test]
fn convert_proof_rejects_wrong_pubkey() {
    // A valid proof bound to pk_A must not verify against an unrelated pubkey.
    //
    // The Fiat-Shamir challenge is c = H(pk, R, ctx, …). The verifier
    // recomputes c using whichever `pk` it's given, so swapping the pubkey
    // at verify time changes the challenge and breaks the algebraic check
    // z·G == R + c·pk. This is the binding property that makes a Convert
    // proof unforgeable against another holder's account.
    //
    // This is exactly the misuse the safe `verify` API is meant to guard
    // against — Custody can branch on the `Result` instead of an `unsafe`
    // status code.

    let (sk_a, pk_a) = keypair::generate().unwrap(); // the real keypair
    let (_, wrong_pk) = keypair::generate().unwrap(); // an unrelated pubkey

    let acc = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let ctx = context::convert(&acc, &iss, 1).unwrap();

    // Generate a *correct* proof for (sk_a, pk_a).
    let proof = prove::convert(&sk_a, &pk_a, &ctx).unwrap();

    // Sanity check: against its own pubkey, the proof verifies. Without this,
    // the rejection below could be hiding a bug where prove silently failed.
    verify::convert(&proof, &pk_a, &ctx).expect("valid proof should verify against its own pubkey");

    // The actual property: same proof, verified against a different pubkey,
    // must be rejected.
    assert!(
        verify::convert(&proof, &wrong_pk, &ctx).is_err(),
        "verifier accepted a valid proof against an unrelated pubkey"
    );
}

// ─────────────────────────────────────────────────────────────────────────
//  Clawback proof — round-trip via the safe verifier
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn clawback_proof_verifies() {
    // Issuer encrypts a "balance" under their own pubkey (this models the
    // issuer's mirror of a holder's balance).
    let (issuer_sk, issuer_pk) = keypair::generate().unwrap();
    let r = encrypt::random_blinding_factor().unwrap();
    let amount = 500u64;
    let mirror = encrypt::encrypt(amount, &issuer_pk, &r).unwrap();

    let acc = AccountId::new([10; 20]);
    let iss = IssuanceId::new([20; 24]);
    let holder = AccountId::new([30; 20]);
    let ctx = context::clawback(&acc, &iss, 7, &holder).unwrap();

    let proof = prove::clawback(&issuer_sk, &issuer_pk, &ctx, amount, &mirror).unwrap();

    verify::clawback(&proof, amount, &issuer_pk, &mirror, &ctx)
        .expect("clawback proof failed verification");
}

// ─────────────────────────────────────────────────────────────────────────
//  ConvertBack proof — round-trip via the safe verifier
// ─────────────────────────────────────────────────────────────────────────

/// Happy-path round trip for `ConfidentialMPTConvertBack` (XLS-0096 §10).
///
/// A holder reveals a withdrawal amount `m` and converts that much of their
/// confidential balance back to public. The 816-byte proof simultaneously
/// establishes three things:
///
///   1. **Ownership** — holder knows the sk for `HolderEncryptionKey`.
///      (compact sigma, 128 B)
///   2. **Commitment linkage** — `BalanceCommitment` represents the same
///      balance as the on-ledger CB_S ciphertext.
///      (same compact sigma; the holder's sk is the link)
///   3. **Non-negative remainder** — `balance − m ≥ 0`, i.e. no overdraft.
///      (Bulletproof range proof, 688 B)
///
/// This test covers only the **positive case**: consistent inputs → valid
/// proof → verifier accepts. It exists to catch FFI-layer breakage — wrong
/// sizes, header drift, parameter-ordering bugs, hash-domain typos. The
/// rejection path (overdraft) lives in
/// [`convert_back_proof_rejects_withdrawal_exceeding_balance`].
///
/// ## Why two independent blinding factors
/// `balance_ciphertext_randomness` is the ElGamal `r` used when encrypting
/// the on-ledger balance ciphertext. `balance_blinding` is the Pedersen `ρ`
/// used in the commitment. They are independent scalars — the proof links
/// the two views of the balance via the holder's sk without requiring
/// `r == ρ`. (Contrast with Send, where a shared `r` IS reused as the
/// Pedersen blinding for `AmountCommitment` per §5.4.)
#[test]
fn convert_back_proof_verifies() {
    let (holder_privkey, holder_pubkey) = keypair::generate().unwrap();

    let current_balance = 1_000u64;
    let withdraw_amount = 250u64;

    // The holder's on-ledger `ConfidentialBalanceSpending` (CB_S):
    // ElGamal encryption of `current_balance` under `holder_pubkey` with
    // fresh randomness.
    let balance_ciphertext_randomness = encrypt::random_blinding_factor().unwrap();
    let balance_ciphertext = encrypt::encrypt(
        current_balance,
        &holder_pubkey,
        &balance_ciphertext_randomness,
    )
    .unwrap();

    // Pedersen commitment to the same balance, but with an INDEPENDENT
    // blinding (conventionally written ρ; unrelated to the ElGamal `r`
    // above). The proof binds these two views of the balance via the
    // sender's secret key — see XLS-0096 §10 / §5.4.
    let balance_blinding = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(current_balance, &balance_blinding).unwrap();

    let holder_account = AccountId::new([5; 20]);
    let issuance_id = IssuanceId::new([6; 24]);
    let context_hash = context::convert_back(
        &holder_account,
        &issuance_id,
        /* sequence */ 1,
        /* version  */ 0,
    )
    .unwrap();

    let proof = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey: &holder_privkey,
        holder_pubkey: &holder_pubkey,
        amount: withdraw_amount,
        current_balance,
        context_hash: &context_hash,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &balance_ciphertext,
    })
    .unwrap();

    verify::convert_back(verify::ConvertBackVerifyParams {
        proof: &proof,
        holder_pubkey: &holder_pubkey,
        balance_ciphertext: &balance_ciphertext,
        balance_commitment: &balance_commitment,
        amount: withdraw_amount,
        context_hash: &context_hash,
    })
    .expect("convert_back proof failed verification");
}

/// Negative-path counterpart to [`convert_back_proof_verifies`].
///
/// Exercises the security property of the embedded Bulletproof range proof:
/// a holder cannot prove that withdrawing more than they hold leaves a
/// non-negative remainder. In the secp256k1 scalar field, `balance − m`
/// when `m > balance` wraps to a value near the group order — far outside
/// the [0, 2^64) range the Bulletproof claims to cover.
///
/// The proof system can refuse the bad witness in either of two places:
///   1. **Prover-side refusal.** The Bulletproof prover detects the out-of-
///      range remainder during bit decomposition and returns an error
///      directly — `prove::convert_back` returns `Err(_)`.
///   2. **Verifier-side rejection.** The prover produces some proof bytes
///      anyway; the verifier rejects them when checking the range proof.
///
/// Either path satisfies the security requirement (no on-chain ConvertBack
/// can drain a confidential balance below zero), so this test asserts that
/// at least one of them fires.
#[test]
fn convert_back_proof_rejects_withdrawal_exceeding_balance() {
    let (holder_privkey, holder_pubkey) = keypair::generate().unwrap();

    let current_balance = 100u64;
    let withdraw_amount = 200u64; // strictly greater than current_balance

    let balance_ciphertext_randomness = encrypt::random_blinding_factor().unwrap();
    let balance_ciphertext = encrypt::encrypt(
        current_balance,
        &holder_pubkey,
        &balance_ciphertext_randomness,
    )
    .unwrap();

    let balance_blinding = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(current_balance, &balance_blinding).unwrap();

    let holder_account = AccountId::new([5; 20]);
    let issuance_id = IssuanceId::new([6; 24]);
    let context_hash = context::convert_back(
        &holder_account,
        &issuance_id,
        /* sequence */ 1,
        /* version  */ 0,
    )
    .unwrap();

    let prove_result = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey: &holder_privkey,
        holder_pubkey: &holder_pubkey,
        amount: withdraw_amount,
        current_balance,
        context_hash: &context_hash,
        balance_commitment: &balance_commitment,
        balance_blinding: &balance_blinding,
        balance_ciphertext: &balance_ciphertext,
    });

    let proof = match prove_result {
        Err(_) => {
            // Path 1: prover refused. Done — security property upheld.
            return;
        }
        Ok(p) => p,
    };

    // Path 2: prover produced bytes despite the inconsistent witness.
    // The verifier MUST reject them.
    assert!(
        verify::convert_back(verify::ConvertBackVerifyParams {
            proof: &proof,
            holder_pubkey: &holder_pubkey,
            balance_ciphertext: &balance_ciphertext,
            balance_commitment: &balance_commitment,
            amount: withdraw_amount,
            context_hash: &context_hash,
        })
        .is_err(),
        "verifier accepted a ConvertBack proof where withdraw > balance"
    );
}

// ─────────────────────────────────────────────────────────────────────────
//  Send proof — round-trip via the safe verifier (3 participants, no auditor)
// ─────────────────────────────────────────────────────────────────────────

/// Builds a consistent 3-participant Send scenario and returns everything a
/// verifier needs. Shared by the Send and range-proof tests.
struct SendScenario {
    proof: SendProof,
    sender_pk: Pubkey,
    recv_pk: Pubkey,
    iss_pk: Pubkey,
    sender_amount_ct: Ciphertext,
    recv_amount_ct: Ciphertext,
    iss_amount_ct: Ciphertext,
    sender_balance_ct: Ciphertext,
    amount_commitment: Commitment,
    balance_commitment: Commitment,
    ctx: ContextHash,
}

fn build_send_scenario() -> SendScenario {
    let (sender_sk, sender_pk) = keypair::generate().unwrap();
    let (_recv_sk, recv_pk) = keypair::generate().unwrap();
    let (_iss_sk, iss_pk) = keypair::generate().unwrap();

    let balance = 1_000u64;
    let amount = 400u64;

    // Sender's on-ledger CB_S.
    let r_balance = encrypt::random_blinding_factor().unwrap();
    let sender_balance_ct = encrypt::encrypt(balance, &sender_pk, &r_balance).unwrap();

    // Pedersen commitment to balance.
    let rho_balance = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(balance, &rho_balance).unwrap();

    // Shared `r` across all three recipient ciphertexts AND the amount commitment.
    let tx_r = encrypt::random_blinding_factor().unwrap();
    let sender_amount_ct = encrypt::encrypt(amount, &sender_pk, &tx_r).unwrap();
    let recv_amount_ct = encrypt::encrypt(amount, &recv_pk, &tx_r).unwrap();
    let iss_amount_ct = encrypt::encrypt(amount, &iss_pk, &tx_r).unwrap();
    let amount_commitment = commit::pedersen(amount, &tx_r).unwrap();

    let snd_addr = AccountId::new([1; 20]);
    let dst_addr = AccountId::new([2; 20]);
    let iss_id = IssuanceId::new([3; 24]);
    let ctx = context::send(&snd_addr, &iss_id, 1, &dst_addr, 0).unwrap();

    let proof = prove::send(prove::SendProofParams {
        sender_privkey: &sender_sk,
        sender_pubkey: &sender_pk,
        amount,
        current_balance: balance,
        tx_blinding_factor: &tx_r,
        context_hash: &ctx,
        amount_commitment: &amount_commitment,
        balance_commitment: &balance_commitment,
        balance_blinding: &rho_balance,
        balance_ciphertext: &sender_balance_ct,
        sender: prove::Participant {
            pubkey: &sender_pk,
            ciphertext: &sender_amount_ct,
        },
        destination: prove::Participant {
            pubkey: &recv_pk,
            ciphertext: &recv_amount_ct,
        },
        issuer: prove::Participant {
            pubkey: &iss_pk,
            ciphertext: &iss_amount_ct,
        },
        auditor: None,
    })
    .unwrap();

    SendScenario {
        proof,
        sender_pk,
        recv_pk,
        iss_pk,
        sender_amount_ct,
        recv_amount_ct,
        iss_amount_ct,
        sender_balance_ct,
        amount_commitment,
        balance_commitment,
        ctx,
    }
}

#[test]
fn send_proof_verifies_three_participants() {
    let s = build_send_scenario();

    verify::send(verify::SendVerifyParams {
        proof: &s.proof,
        sender: prove::Participant {
            pubkey: &s.sender_pk,
            ciphertext: &s.sender_amount_ct,
        },
        destination: prove::Participant {
            pubkey: &s.recv_pk,
            ciphertext: &s.recv_amount_ct,
        },
        issuer: prove::Participant {
            pubkey: &s.iss_pk,
            ciphertext: &s.iss_amount_ct,
        },
        auditor: None,
        sender_spending_ciphertext: &s.sender_balance_ct,
        amount_commitment: &s.amount_commitment,
        balance_commitment: &s.balance_commitment,
        context_hash: &s.ctx,
    })
    .expect("send proof failed verification");
}

/// The 754-byte range component sits after the 192-byte compact sigma inside
/// the 946-byte Send proof. Verifying that slice on its own exercises the
/// lower-level [`verify::send_range_proof`] wrapper against the C oracle.
#[test]
fn send_range_proof_verifies_extracted_subproof() {
    let s = build_send_scenario();
    let range = &s.proof.as_bytes()[192..]; // 754-byte double Bulletproof

    verify::send_range_proof(range, &s.amount_commitment, &s.balance_commitment, &s.ctx)
        .expect("extracted send range proof failed verification");
}

// ─────────────────────────────────────────────────────────────────────────
//  Revealed-amount consistency
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn revealed_amount_verifies_matching_ciphertexts() {
    let (_h_sk, holder_pk) = keypair::generate().unwrap();
    let (_i_sk, issuer_pk) = keypair::generate().unwrap();
    let (_a_sk, auditor_pk) = keypair::generate().unwrap();

    let amount = 777u64;
    // The SAME ElGamal randomness `r` is used for every participant's
    // ciphertext — that shared scalar is what `revealed_amount` checks.
    let r = encrypt::random_blinding_factor().unwrap();
    let holder_ct = encrypt::encrypt(amount, &holder_pk, &r).unwrap();
    let issuer_ct = encrypt::encrypt(amount, &issuer_pk, &r).unwrap();
    let auditor_ct = encrypt::encrypt(amount, &auditor_pk, &r).unwrap();

    let holder = prove::Participant {
        pubkey: &holder_pk,
        ciphertext: &holder_ct,
    };
    let issuer = prove::Participant {
        pubkey: &issuer_pk,
        ciphertext: &issuer_ct,
    };
    let auditor = prove::Participant {
        pubkey: &auditor_pk,
        ciphertext: &auditor_ct,
    };

    verify::revealed_amount(amount, &r, holder, issuer, Some(auditor))
        .expect("matching ciphertexts should verify");

    // A different claimed amount must be rejected.
    assert!(
        verify::revealed_amount(amount + 1, &r, holder, issuer, Some(auditor)).is_err(),
        "verifier accepted a wrong revealed amount"
    );
}

// ─────────────────────────────────────────────────────────────────────────
//  Input-validation guards on the lower-level verifiers
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn send_range_proof_rejects_wrong_length() {
    let r = encrypt::random_blinding_factor().unwrap();
    let c = commit::pedersen(1, &r).unwrap();
    let ctx = context::send(
        &AccountId::new([1; 20]),
        &IssuanceId::new([2; 24]),
        1,
        &AccountId::new([3; 20]),
        0,
    )
    .unwrap();

    let err = verify::send_range_proof(&[0u8; 10], &c, &c, &ctx).unwrap_err();
    assert!(
        matches!(err, Error::Invariant(_)),
        "expected an Invariant error for a wrong-length range proof, got {err:?}"
    );
}

#[test]
fn aggregated_bulletproof_rejects_empty_commitments() {
    let ctx = context::convert(&AccountId::new([1; 20]), &IssuanceId::new([2; 24]), 1).unwrap();
    let err = verify::aggregated_bulletproof(&[0u8; 100], &[], &ctx).unwrap_err();
    assert!(
        matches!(err, Error::Invariant(_)),
        "expected an Invariant error for empty commitments, got {err:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════
//  Negative / edge cases mirrored from upstream tests/test_mpt_utility.cpp
//
//  Upstream exercises, for each proof type: a corrupted proof (first byte
//  XORed), a zeroed context hash, and wrong public inputs. We reproduce those
//  against the safe `verify` API so a regression in our FFI plumbing (wrong
//  buffer size, parameter order, ctx binding) surfaces as a test failure.
//
//  Upstream's `n=2` prover/verifier rejections have no analogue here: the
//  `SendProofParams`/`SendVerifyParams` types require sender + destination +
//  issuer, so fewer than three participants cannot be expressed.
// ═════════════════════════════════════════════════════════════════════════

/// Flip the first byte of any proof's bytes — the upstream "corrupted proof"
/// mutation (`proof[0] ^= 0xFF`).
fn flip_first_byte<const N: usize>(bytes: &[u8; N]) -> [u8; N] {
    let mut out = *bytes;
    out[0] ^= 0xFF;
    out
}

const ZERO_CTX: ContextHash = ContextHash::new([0u8; 32]);

// ── Encrypt/decrypt edge amounts (upstream: 0, 1, 1000) ──────────────────

#[test]
fn encrypt_decrypt_edge_amounts() {
    let (sk, pk) = keypair::generate().unwrap();
    for amount in [0u64, 1, 1000] {
        let r = encrypt::random_blinding_factor().unwrap();
        let ct = encrypt::encrypt(amount, &pk, &r).unwrap();
        assert_eq!(
            encrypt::decrypt(&ct, &sk).unwrap(),
            amount,
            "round-trip failed for amount {amount}"
        );
    }
}

// ── Convert negatives ────────────────────────────────────────────────────

#[test]
fn convert_proof_rejects_corrupted_proof() {
    let (sk, pk) = keypair::generate().unwrap();
    let ctx = context::convert(&AccountId::new([1; 20]), &IssuanceId::new([2; 24]), 1).unwrap();
    let proof = prove::convert(&sk, &pk, &ctx).unwrap();

    let corrupted = ConvertProof::new(flip_first_byte(proof.as_bytes()));
    assert!(
        verify::convert(&corrupted, &pk, &ctx).is_err(),
        "verifier accepted a corrupted convert proof"
    );
}

#[test]
fn convert_proof_rejects_wrong_context_hash() {
    let (sk, pk) = keypair::generate().unwrap();
    let ctx = context::convert(&AccountId::new([1; 20]), &IssuanceId::new([2; 24]), 1).unwrap();
    let proof = prove::convert(&sk, &pk, &ctx).unwrap();

    assert!(
        verify::convert(&proof, &pk, &ZERO_CTX).is_err(),
        "verifier accepted a convert proof against a zeroed context hash"
    );
}

// ── Send: n=4 (with auditor) + negatives ─────────────────────────────────

#[test]
fn send_proof_verifies_four_participants_with_auditor() {
    let (sender_sk, sender_pk) = keypair::generate().unwrap();
    let (_recv_sk, recv_pk) = keypair::generate().unwrap();
    let (_iss_sk, iss_pk) = keypair::generate().unwrap();
    let (_aud_sk, aud_pk) = keypair::generate().unwrap();

    let balance = 1_000u64;
    let amount = 400u64;

    let r_balance = encrypt::random_blinding_factor().unwrap();
    let sender_balance_ct = encrypt::encrypt(balance, &sender_pk, &r_balance).unwrap();
    let rho_balance = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(balance, &rho_balance).unwrap();

    let tx_r = encrypt::random_blinding_factor().unwrap();
    let sender_amount_ct = encrypt::encrypt(amount, &sender_pk, &tx_r).unwrap();
    let recv_amount_ct = encrypt::encrypt(amount, &recv_pk, &tx_r).unwrap();
    let iss_amount_ct = encrypt::encrypt(amount, &iss_pk, &tx_r).unwrap();
    let aud_amount_ct = encrypt::encrypt(amount, &aud_pk, &tx_r).unwrap();
    let amount_commitment = commit::pedersen(amount, &tx_r).unwrap();

    let ctx = context::send(
        &AccountId::new([1; 20]),
        &IssuanceId::new([3; 24]),
        1,
        &AccountId::new([2; 20]),
        0,
    )
    .unwrap();

    let sender = prove::Participant {
        pubkey: &sender_pk,
        ciphertext: &sender_amount_ct,
    };
    let destination = prove::Participant {
        pubkey: &recv_pk,
        ciphertext: &recv_amount_ct,
    };
    let issuer = prove::Participant {
        pubkey: &iss_pk,
        ciphertext: &iss_amount_ct,
    };
    let auditor = prove::Participant {
        pubkey: &aud_pk,
        ciphertext: &aud_amount_ct,
    };

    let proof = prove::send(prove::SendProofParams {
        sender_privkey: &sender_sk,
        sender_pubkey: &sender_pk,
        amount,
        current_balance: balance,
        tx_blinding_factor: &tx_r,
        context_hash: &ctx,
        amount_commitment: &amount_commitment,
        balance_commitment: &balance_commitment,
        balance_blinding: &rho_balance,
        balance_ciphertext: &sender_balance_ct,
        sender,
        destination,
        issuer,
        auditor: Some(auditor),
    })
    .unwrap();

    verify::send(verify::SendVerifyParams {
        proof: &proof,
        sender,
        destination,
        issuer,
        auditor: Some(auditor),
        sender_spending_ciphertext: &sender_balance_ct,
        amount_commitment: &amount_commitment,
        balance_commitment: &balance_commitment,
        context_hash: &ctx,
    })
    .expect("send proof (n=4, with auditor) failed verification");
}

/// Build the all-correct `SendVerifyParams` for a 3-participant scenario.
/// Individual negative tests then override one field.
fn send_verify_params<'a>(s: &'a SendScenario) -> verify::SendVerifyParams<'a> {
    verify::SendVerifyParams {
        proof: &s.proof,
        sender: prove::Participant {
            pubkey: &s.sender_pk,
            ciphertext: &s.sender_amount_ct,
        },
        destination: prove::Participant {
            pubkey: &s.recv_pk,
            ciphertext: &s.recv_amount_ct,
        },
        issuer: prove::Participant {
            pubkey: &s.iss_pk,
            ciphertext: &s.iss_amount_ct,
        },
        auditor: None,
        sender_spending_ciphertext: &s.sender_balance_ct,
        amount_commitment: &s.amount_commitment,
        balance_commitment: &s.balance_commitment,
        context_hash: &s.ctx,
    }
}

#[test]
fn send_proof_rejects_corrupted_proof() {
    let s = build_send_scenario();
    let corrupted = SendProof::new(flip_first_byte(s.proof.as_bytes()));
    let mut params = send_verify_params(&s);
    params.proof = &corrupted;
    assert!(
        verify::send(params).is_err(),
        "verifier accepted a corrupted send proof"
    );
}

#[test]
fn send_proof_rejects_wrong_context_hash() {
    let s = build_send_scenario();
    let mut params = send_verify_params(&s);
    params.context_hash = &ZERO_CTX;
    assert!(
        verify::send(params).is_err(),
        "verifier accepted a send proof against a zeroed context hash"
    );
}

#[test]
fn send_proof_rejects_wrong_amount_commitment() {
    let s = build_send_scenario();
    let wrong = commit::pedersen(123_456, &encrypt::random_blinding_factor().unwrap()).unwrap();
    let mut params = send_verify_params(&s);
    params.amount_commitment = &wrong;
    assert!(
        verify::send(params).is_err(),
        "verifier accepted a send proof with a wrong amount commitment"
    );
}

#[test]
fn send_proof_rejects_wrong_balance_commitment() {
    let s = build_send_scenario();
    let wrong = commit::pedersen(123_456, &encrypt::random_blinding_factor().unwrap()).unwrap();
    let mut params = send_verify_params(&s);
    params.balance_commitment = &wrong;
    assert!(
        verify::send(params).is_err(),
        "verifier accepted a send proof with a wrong balance commitment"
    );
}

#[test]
fn send_proof_rejects_wrong_balance_ciphertext() {
    let s = build_send_scenario();
    let wrong = encrypt::encrypt(
        999,
        &s.sender_pk,
        &encrypt::random_blinding_factor().unwrap(),
    )
    .unwrap();
    let mut params = send_verify_params(&s);
    params.sender_spending_ciphertext = &wrong;
    assert!(
        verify::send(params).is_err(),
        "verifier accepted a send proof with a wrong spending-balance ciphertext"
    );
}

// ── ConvertBack negatives ─────────────────────────────────────────────────

struct ConvertBackScenario {
    proof: ConvertBackProof,
    holder_pubkey: Pubkey,
    balance_ciphertext: Ciphertext,
    balance_commitment: Commitment,
    withdraw_amount: u64,
    ctx: ContextHash,
}

fn build_convert_back_scenario() -> ConvertBackScenario {
    let (holder_privkey, holder_pubkey) = keypair::generate().unwrap();
    let current_balance = 5_000u64;
    let withdraw_amount = 1_000u64;

    let r = encrypt::random_blinding_factor().unwrap();
    let balance_ciphertext = encrypt::encrypt(current_balance, &holder_pubkey, &r).unwrap();
    let rho = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(current_balance, &rho).unwrap();
    let ctx =
        context::convert_back(&AccountId::new([5; 20]), &IssuanceId::new([6; 24]), 1, 0).unwrap();

    let proof = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey: &holder_privkey,
        holder_pubkey: &holder_pubkey,
        amount: withdraw_amount,
        current_balance,
        context_hash: &ctx,
        balance_commitment: &balance_commitment,
        balance_blinding: &rho,
        balance_ciphertext: &balance_ciphertext,
    })
    .unwrap();

    ConvertBackScenario {
        proof,
        holder_pubkey,
        balance_ciphertext,
        balance_commitment,
        withdraw_amount,
        ctx,
    }
}

fn convert_back_verify_params<'a>(
    s: &'a ConvertBackScenario,
) -> verify::ConvertBackVerifyParams<'a> {
    verify::ConvertBackVerifyParams {
        proof: &s.proof,
        holder_pubkey: &s.holder_pubkey,
        balance_ciphertext: &s.balance_ciphertext,
        balance_commitment: &s.balance_commitment,
        amount: s.withdraw_amount,
        context_hash: &s.ctx,
    }
}

#[test]
fn convert_back_proof_rejects_corrupted_proof() {
    let s = build_convert_back_scenario();
    let corrupted = ConvertBackProof::new(flip_first_byte(s.proof.as_bytes()));
    let mut params = convert_back_verify_params(&s);
    params.proof = &corrupted;
    assert!(
        verify::convert_back(params).is_err(),
        "verifier accepted a corrupted convert_back proof"
    );
}

#[test]
fn convert_back_proof_rejects_wrong_context_hash() {
    let s = build_convert_back_scenario();
    let mut params = convert_back_verify_params(&s);
    params.context_hash = &ZERO_CTX;
    assert!(
        verify::convert_back(params).is_err(),
        "verifier accepted a convert_back proof against a zeroed context hash"
    );
}

#[test]
fn convert_back_proof_rejects_wrong_balance_commitment() {
    let s = build_convert_back_scenario();
    let wrong = commit::pedersen(123_456, &encrypt::random_blinding_factor().unwrap()).unwrap();
    let mut params = convert_back_verify_params(&s);
    params.balance_commitment = &wrong;
    assert!(
        verify::convert_back(params).is_err(),
        "verifier accepted a convert_back proof with a wrong balance commitment"
    );
}

#[test]
fn convert_back_proof_rejects_wrong_balance_ciphertext() {
    let s = build_convert_back_scenario();
    let wrong = encrypt::encrypt(
        999,
        &s.holder_pubkey,
        &encrypt::random_blinding_factor().unwrap(),
    )
    .unwrap();
    let mut params = convert_back_verify_params(&s);
    params.balance_ciphertext = &wrong;
    assert!(
        verify::convert_back(params).is_err(),
        "verifier accepted a convert_back proof with a wrong balance ciphertext"
    );
}

// ── Clawback negatives ─────────────────────────────────────────────────────

struct ClawbackScenario {
    proof: ClawbackProof,
    amount: u64,
    issuer_pk: Pubkey,
    mirror: Ciphertext,
    ctx: ContextHash,
}

fn build_clawback_scenario() -> ClawbackScenario {
    let (issuer_sk, issuer_pk) = keypair::generate().unwrap();
    let r = encrypt::random_blinding_factor().unwrap();
    let amount = 500u64;
    let mirror = encrypt::encrypt(amount, &issuer_pk, &r).unwrap();
    let ctx = context::clawback(
        &AccountId::new([10; 20]),
        &IssuanceId::new([20; 24]),
        7,
        &AccountId::new([30; 20]),
    )
    .unwrap();
    let proof = prove::clawback(&issuer_sk, &issuer_pk, &ctx, amount, &mirror).unwrap();

    ClawbackScenario {
        proof,
        amount,
        issuer_pk,
        mirror,
        ctx,
    }
}

#[test]
fn clawback_proof_rejects_corrupted_proof() {
    let s = build_clawback_scenario();
    let corrupted = ClawbackProof::new(flip_first_byte(s.proof.as_bytes()));
    assert!(
        verify::clawback(&corrupted, s.amount, &s.issuer_pk, &s.mirror, &s.ctx).is_err(),
        "verifier accepted a corrupted clawback proof"
    );
}

#[test]
fn clawback_proof_rejects_wrong_context_hash() {
    let s = build_clawback_scenario();
    assert!(
        verify::clawback(&s.proof, s.amount, &s.issuer_pk, &s.mirror, &ZERO_CTX).is_err(),
        "verifier accepted a clawback proof against a zeroed context hash"
    );
}

#[test]
fn clawback_proof_rejects_wrong_amount() {
    let s = build_clawback_scenario();
    // Upstream verifies 500-byte proof against amount 999.
    assert!(
        verify::clawback(&s.proof, s.amount + 499, &s.issuer_pk, &s.mirror, &s.ctx).is_err(),
        "verifier accepted a clawback proof against the wrong amount"
    );
}

#[test]
fn clawback_proof_rejects_wrong_ciphertext() {
    let s = build_clawback_scenario();
    // Same amount, but a ciphertext made with a different blinding factor.
    let wrong = encrypt::encrypt(
        s.amount,
        &s.issuer_pk,
        &encrypt::random_blinding_factor().unwrap(),
    )
    .unwrap();
    assert!(
        verify::clawback(&s.proof, s.amount, &s.issuer_pk, &wrong, &s.ctx).is_err(),
        "verifier accepted a clawback proof against a wrong ciphertext"
    );
}
