//! Integration tests for the safe wrapper.
//!
//! Each test exercises one slice of the API end-to-end. Where the safe
//! wrapper produces a proof, we verify it via the corresponding `mpt_verify_*`
//! FFI function — the C verifier is the oracle for "did our prove call
//! produce something rippled would accept?"

use mpt_crypto::*;
use mpt_crypto_sys as sys;

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
    assert_eq!(ct1.as_bytes(), ct2.as_bytes(),
        "same (m, r, pk) should produce identical ciphertext");
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
    assert!(s.contains("redacted"), "Privkey Debug must not expose bytes");
    // The hex of any bit of the actual key should not appear in the Debug output.
    let hex: String = sk.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    assert!(!s.contains(&hex[0..8]),
        "Privkey Debug output appears to contain key bytes");
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

    let h_convert      = context::convert(&acc, &iss, seq).unwrap();
    let h_convert_back = context::convert_back(&acc, &iss, seq, ver).unwrap();
    let h_send         = context::send(&acc, &iss, seq, &dst, ver).unwrap();
    let h_clawback     = context::clawback(&acc, &iss, seq, &dst).unwrap();

    let all = [h_convert, h_convert_back, h_send, h_clawback];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j],
                "context hashes for different tx types collided ({i} vs {j})");
        }
    }
}

#[test]
fn send_context_hash_binds_to_destination() {
    let snd = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let d1  = AccountId::new([3; 20]);
    let d2  = AccountId::new([4; 20]);
    let h1 = context::send(&snd, &iss, 1, &d1, 0).unwrap();
    let h2 = context::send(&snd, &iss, 1, &d2, 0).unwrap();
    assert_ne!(h1, h2, "Send proof must be unforgeable across destinations");
}

// ─────────────────────────────────────────────────────────────────────────
//  Convert proof — Schnorr PoK round-trip via the C verifier
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn convert_proof_verifies() {
    let (sk, pk) = keypair::generate().unwrap();
    let acc = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let ctx = context::convert(&acc, &iss, 1).unwrap();

    let proof = prove::convert(&sk, &pk, &ctx).unwrap();

    // SAFETY: arguments are fixed-size buffers matching the C contract.
    let rc = unsafe {
        sys::mpt_verify_convert_proof(
            proof.as_bytes().as_ptr(),
            pk.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0, "convert proof failed C-side verification");
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

    let (sk_a, pk_a)   = keypair::generate().unwrap();   // the real keypair
    let (_, wrong_pk)  = keypair::generate().unwrap();   // an unrelated pubkey

    let acc = AccountId::new([1; 20]);
    let iss = IssuanceId::new([2; 24]);
    let ctx = context::convert(&acc, &iss, 1).unwrap();

    // Generate a *correct* proof for (sk_a, pk_a).
    let proof = prove::convert(&sk_a, &pk_a, &ctx).unwrap();

    // Sanity check: against its own pubkey, the proof verifies. Without this,
    // the rejection below could be hiding a bug where prove silently failed.
    // SAFETY: fixed-size buffers per the FFI contract.
    let rc_ok = unsafe {
        sys::mpt_verify_convert_proof(
            proof.as_bytes().as_ptr(),
            pk_a.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
        )
    };
    assert_eq!(rc_ok, 0, "valid proof should verify against its own pubkey");

    // The actual property: same proof, verified against a different pubkey,
    // must be rejected.
    let rc_wrong = unsafe {
        sys::mpt_verify_convert_proof(
            proof.as_bytes().as_ptr(),
            wrong_pk.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
        )
    };
    assert_ne!(rc_wrong, 0,
        "verifier accepted a valid proof against an unrelated pubkey");
}

// ─────────────────────────────────────────────────────────────────────────
//  Clawback proof — round-trip via the C verifier
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn clawback_proof_verifies() {
    // Issuer encrypts a "balance" under their own pubkey (this models the
    // issuer's mirror of a holder's balance).
    let (issuer_sk, issuer_pk) = keypair::generate().unwrap();
    let r = encrypt::random_blinding_factor().unwrap();
    let amount = 500u64;
    let mirror = encrypt::encrypt(amount, &issuer_pk, &r).unwrap();

    let acc    = AccountId::new([10; 20]);
    let iss    = IssuanceId::new([20; 24]);
    let holder = AccountId::new([30; 20]);
    let ctx    = context::clawback(&acc, &iss, 7, &holder).unwrap();

    let proof = prove::clawback(&issuer_sk, &issuer_pk, &ctx, amount, &mirror).unwrap();

    // SAFETY: fixed-size buffers per FFI contract.
    let rc = unsafe {
        sys::mpt_verify_clawback_proof(
            proof.as_bytes().as_ptr(),
            amount,
            issuer_pk.as_bytes().as_ptr(),
            mirror.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0, "clawback proof failed verification");
}

// ─────────────────────────────────────────────────────────────────────────
//  ConvertBack proof — round-trip via the C verifier
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
    let withdraw_amount =   250u64;

    // The holder's on-ledger `ConfidentialBalanceSpending` (CB_S):
    // ElGamal encryption of `current_balance` under `holder_pubkey` with
    // fresh randomness.
    let balance_ciphertext_randomness = encrypt::random_blinding_factor().unwrap();
    let balance_ciphertext = encrypt::encrypt(
        current_balance,
        &holder_pubkey,
        &balance_ciphertext_randomness,
    ).unwrap();

    // Pedersen commitment to the same balance, but with an INDEPENDENT
    // blinding (conventionally written ρ; unrelated to the ElGamal `r`
    // above). The proof binds these two views of the balance via the
    // sender's secret key — see XLS-0096 §10 / §5.4.
    let balance_blinding   = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(current_balance, &balance_blinding).unwrap();

    let holder_account = AccountId::new([5; 20]);
    let issuance_id    = IssuanceId::new([6; 24]);
    let context_hash   = context::convert_back(
        &holder_account,
        &issuance_id,
        /* sequence */ 1,
        /* version  */ 0,
    ).unwrap();

    let proof = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey:     &holder_privkey,
        holder_pubkey:      &holder_pubkey,
        amount:             withdraw_amount,
        current_balance,
        context_hash:       &context_hash,
        balance_commitment: &balance_commitment,
        balance_blinding:   &balance_blinding,
        balance_ciphertext: &balance_ciphertext,
    }).unwrap();

    // SAFETY: fixed-size buffers per the FFI contract.
    let rc = unsafe {
        sys::mpt_verify_convert_back_proof(
            proof.as_bytes().as_ptr(),
            holder_pubkey.as_bytes().as_ptr(),
            balance_ciphertext.as_bytes().as_ptr(),
            balance_commitment.as_bytes().as_ptr(),
            withdraw_amount,
            context_hash.as_bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0, "convert_back proof failed verification");
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
///      anyway; the C verifier rejects them when checking the range proof.
///
/// Either path satisfies the security requirement (no on-chain ConvertBack
/// can drain a confidential balance below zero), so this test asserts that
/// at least one of them fires.
#[test]
fn convert_back_proof_rejects_withdrawal_exceeding_balance() {
    let (holder_privkey, holder_pubkey) = keypair::generate().unwrap();

    let current_balance = 100u64;
    let withdraw_amount = 200u64;   // strictly greater than current_balance

    let balance_ciphertext_randomness = encrypt::random_blinding_factor().unwrap();
    let balance_ciphertext = encrypt::encrypt(
        current_balance,
        &holder_pubkey,
        &balance_ciphertext_randomness,
    ).unwrap();

    let balance_blinding   = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(current_balance, &balance_blinding).unwrap();

    let holder_account = AccountId::new([5; 20]);
    let issuance_id    = IssuanceId::new([6; 24]);
    let context_hash   = context::convert_back(
        &holder_account,
        &issuance_id,
        /* sequence */ 1,
        /* version  */ 0,
    ).unwrap();

    let prove_result = prove::convert_back(prove::ConvertBackProofParams {
        holder_privkey:     &holder_privkey,
        holder_pubkey:      &holder_pubkey,
        amount:             withdraw_amount,
        current_balance,
        context_hash:       &context_hash,
        balance_commitment: &balance_commitment,
        balance_blinding:   &balance_blinding,
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
    // SAFETY: fixed-size buffers per the FFI contract.
    let rc = unsafe {
        sys::mpt_verify_convert_back_proof(
            proof.as_bytes().as_ptr(),
            holder_pubkey.as_bytes().as_ptr(),
            balance_ciphertext.as_bytes().as_ptr(),
            balance_commitment.as_bytes().as_ptr(),
            withdraw_amount,
            context_hash.as_bytes().as_ptr(),
        )
    };
    assert_ne!(rc, 0,
        "verifier accepted a ConvertBack proof where withdraw > balance");
}

// ─────────────────────────────────────────────────────────────────────────
//  Send proof — round-trip via the C verifier (3 participants, no auditor)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn send_proof_verifies_three_participants() {
    let (sender_sk, sender_pk) = keypair::generate().unwrap();
    let (_recv_sk,  recv_pk)   = keypair::generate().unwrap();
    let (_iss_sk,   iss_pk)    = keypair::generate().unwrap();

    let balance = 1_000u64;
    let amount  =   400u64;

    // Sender's on-ledger CB_S.
    let r_balance = encrypt::random_blinding_factor().unwrap();
    let sender_balance_ct = encrypt::encrypt(balance, &sender_pk, &r_balance).unwrap();

    // Pedersen commitment to balance.
    let rho_balance = encrypt::random_blinding_factor().unwrap();
    let balance_commitment = commit::pedersen(balance, &rho_balance).unwrap();

    // Shared `r` across all three recipient ciphertexts AND the amount commitment.
    let tx_r = encrypt::random_blinding_factor().unwrap();
    let sender_amount_ct = encrypt::encrypt(amount, &sender_pk, &tx_r).unwrap();
    let recv_amount_ct   = encrypt::encrypt(amount, &recv_pk,   &tx_r).unwrap();
    let iss_amount_ct    = encrypt::encrypt(amount, &iss_pk,    &tx_r).unwrap();
    let amount_commitment = commit::pedersen(amount, &tx_r).unwrap();

    let snd_addr = AccountId::new([1; 20]);
    let dst_addr = AccountId::new([2; 20]);
    let iss_id   = IssuanceId::new([3; 24]);
    let ctx = context::send(&snd_addr, &iss_id, 1, &dst_addr, 0).unwrap();

    let sender_part = prove::Participant { pubkey: &sender_pk, ciphertext: &sender_amount_ct };
    let recv_part   = prove::Participant { pubkey: &recv_pk,   ciphertext: &recv_amount_ct };
    let iss_part    = prove::Participant { pubkey: &iss_pk,    ciphertext: &iss_amount_ct };

    let proof = prove::send(prove::SendProofParams {
        sender_privkey:     &sender_sk,
        sender_pubkey:      &sender_pk,
        amount,
        current_balance:    balance,
        tx_blinding_factor: &tx_r,
        context_hash:       &ctx,
        amount_commitment:  &amount_commitment,
        balance_commitment: &balance_commitment,
        balance_blinding:   &rho_balance,
        balance_ciphertext: &sender_balance_ct,
        sender:             sender_part,
        destination:        recv_part,
        issuer:             iss_part,
        auditor:            None,
    }).unwrap();

    // The C verifier expects participants in the same order we passed them.
    let participants_for_verify = [
        sys::mpt_confidential_participant {
            pubkey: *sender_pk.as_bytes(), ciphertext: *sender_amount_ct.as_bytes(),
        },
        sys::mpt_confidential_participant {
            pubkey: *recv_pk.as_bytes(),   ciphertext: *recv_amount_ct.as_bytes(),
        },
        sys::mpt_confidential_participant {
            pubkey: *iss_pk.as_bytes(),    ciphertext: *iss_amount_ct.as_bytes(),
        },
    ];

    let rc = unsafe {
        sys::mpt_verify_send_proof(
            proof.as_bytes().as_ptr(),
            participants_for_verify.as_ptr(),
            3,
            sender_balance_ct.as_bytes().as_ptr(),
            amount_commitment.as_bytes().as_ptr(),
            balance_commitment.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
        )
    };
    assert_eq!(rc, 0, "send proof failed verification");
}
