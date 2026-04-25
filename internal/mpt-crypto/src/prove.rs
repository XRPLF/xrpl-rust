//! Per-transaction-type proof generation.
//!
//! These are the four user-facing prove functions matching the four
//! proof-bearing transaction types in XLS-0096:
//!
//! | Function | Output | XLS-0096 reference |
//! |---|---|---|
//! | [`convert`]      | [`ConvertProof`]     (64 B Schnorr PoK) | §7.2 |
//! | [`send`]         | [`SendProof`]        (946 B composite)  | §8.2 / §5.4 |
//! | [`convert_back`] | [`ConvertBackProof`] (816 B composite)  | §10.3 |
//! | [`clawback`]     | [`ClawbackProof`]    (64 B compact sigma) | §11.2 |
//!
//! The `*Params` structs collect the long argument lists into struct-init
//! syntax, which makes call sites self-documenting and avoids parameter-
//! ordering bugs.

use crate::{
    Error, Result,
    types::{
        BlindingFactor, Ciphertext, ClawbackProof, Commitment, ContextHash, ConvertBackProof,
        ConvertProof, Privkey, Pubkey, SendProof,
    },
};
use mpt_crypto_sys as sys;

/// One participant in a confidential transfer: their public key plus the
/// ciphertext of the transfer amount under that key.
#[derive(Debug, Clone, Copy)]
pub struct Participant<'a> {
    pub pubkey:     &'a Pubkey,
    pub ciphertext: &'a Ciphertext,
}

// ─────────────────────────────────────────────────────────────────────────
//  Convert: 64-byte Schnorr Proof of Knowledge
// ─────────────────────────────────────────────────────────────────────────

/// Generates the 64-byte Schnorr Proof of Knowledge required at first
/// `ConfidentialMPTConvert` (when registering the holder's `pk`).
///
/// Returns `Err(NonZeroRc(_))` if the underlying primitive fails — typically
/// indicates a malformed pubkey.
pub fn convert(privkey: &Privkey, pubkey: &Pubkey, ctx: &ContextHash) -> Result<ConvertProof> {
    let mut out = [0u8; 64];
    // SAFETY: pointer arguments target fixed-size buffers matching the FFI
    //         contract (33-byte pubkey, 32-byte privkey, 32-byte ctx hash,
    //         64-byte output).
    let rc = unsafe {
        sys::mpt_get_convert_proof(
            pubkey.as_bytes().as_ptr(),
            privkey.as_bytes().as_ptr(),
            ctx.as_bytes().as_ptr(),
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ConvertProof::new(out))
}

// ─────────────────────────────────────────────────────────────────────────
//  Send: 946-byte composite (192 B compact sigma + 754 B aggregated BP)
// ─────────────────────────────────────────────────────────────────────────

/// Inputs to [`send`]. Lots of fields because the Send ZK statement has
/// many witnesses.
///
/// **Critical invariant** (XLS-0096 §5.4): `tx_blinding_factor` is the same
/// scalar used both as the ElGamal randomness `r` for *all* participant
/// ciphertexts AND as the Pedersen blinding for `amount_commitment`. The
/// compact sigma collapses the amount-linkage proof into the main sigma by
/// reusing this scalar. Pass an independent value here and verification will
/// fail.
///
/// `balance_ciphertext` must be the holder's `CB_S` **as it currently lives
/// on the ledger** (not a fresh encryption of `current_balance`). The proof
/// links the new `balance_commitment` to this on-chain ciphertext via the
/// holder's secret key.
pub struct SendProofParams<'a> {
    pub sender_privkey:     &'a Privkey,
    pub sender_pubkey:      &'a Pubkey,
    pub amount:             u64,
    pub current_balance:    u64,
    pub tx_blinding_factor: &'a BlindingFactor,
    pub context_hash:       &'a ContextHash,
    pub amount_commitment:  &'a Commitment,
    pub balance_commitment: &'a Commitment,
    pub balance_blinding:   &'a BlindingFactor,
    pub balance_ciphertext: &'a Ciphertext,
    pub sender:             Participant<'a>,
    pub destination:        Participant<'a>,
    pub issuer:             Participant<'a>,
    pub auditor:            Option<Participant<'a>>,
}

/// Generates the 946-byte `ConfidentialMPTSend` proof bundle.
pub fn send(p: SendProofParams<'_>) -> Result<SendProof> {
    // The C side expects a contiguous array of `mpt_confidential_participant`.
    // We build it on the stack — at most 4 entries.
    let mut participants = [
        sys::mpt_confidential_participant { pubkey: [0; 33], ciphertext: [0; 66] }; 4
    ];

    fn fill(slot: &mut sys::mpt_confidential_participant, p: Participant<'_>) {
        slot.pubkey     = *p.pubkey.as_bytes();
        slot.ciphertext = *p.ciphertext.as_bytes();
    }

    fill(&mut participants[0], p.sender);
    fill(&mut participants[1], p.destination);
    fill(&mut participants[2], p.issuer);

    let n = if let Some(aud) = p.auditor {
        fill(&mut participants[3], aud);
        4
    } else {
        3
    };

    let balance_params = sys::mpt_pedersen_proof_params {
        pedersen_commitment: *p.balance_commitment.as_bytes(),
        amount:              p.current_balance,
        ciphertext:          *p.balance_ciphertext.as_bytes(),
        blinding_factor:     *p.balance_blinding.as_bytes(),
    };

    let mut out = [0u8; 946];
    let mut out_len: usize = 946;

    // SAFETY: All buffers are sized to the FFI contract; participants array
    //         length matches the `n` we pass. `&balance_params` is borrowed
    //         exclusively for the call.
    let rc = unsafe {
        sys::mpt_get_confidential_send_proof(
            p.sender_privkey.as_bytes().as_ptr(),
            p.sender_pubkey.as_bytes().as_ptr(),
            p.amount,
            participants.as_ptr(),
            n,
            p.tx_blinding_factor.as_bytes().as_ptr(),
            p.context_hash.as_bytes().as_ptr(),
            p.amount_commitment.as_bytes().as_ptr(),
            &balance_params,
            out.as_mut_ptr(),
            &mut out_len,
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    if out_len != 946 {
        return Err(Error::Invariant("send proof had unexpected length"));
    }
    Ok(SendProof::new(out))
}

// ─────────────────────────────────────────────────────────────────────────
//  ConvertBack: 816-byte composite (128 B compact sigma + 688 B BP)
// ─────────────────────────────────────────────────────────────────────────

/// Inputs to [`convert_back`].
///
/// `balance_ciphertext` must be the holder's on-ledger `CB_S` ciphertext —
/// same constraint as in [`SendProofParams`]. `amount` is the publicly
/// revealed plaintext withdrawal amount.
pub struct ConvertBackProofParams<'a> {
    pub holder_privkey:     &'a Privkey,
    pub holder_pubkey:      &'a Pubkey,
    pub amount:             u64,
    pub current_balance:    u64,
    pub context_hash:       &'a ContextHash,
    pub balance_commitment: &'a Commitment,
    pub balance_blinding:   &'a BlindingFactor,
    pub balance_ciphertext: &'a Ciphertext,
}

/// Generates the 816-byte `ConfidentialMPTConvertBack` proof bundle.
pub fn convert_back(p: ConvertBackProofParams<'_>) -> Result<ConvertBackProof> {
    let params = sys::mpt_pedersen_proof_params {
        pedersen_commitment: *p.balance_commitment.as_bytes(),
        amount:              p.current_balance,
        ciphertext:          *p.balance_ciphertext.as_bytes(),
        blinding_factor:     *p.balance_blinding.as_bytes(),
    };
    let mut out = [0u8; 816];
    // SAFETY: see `send`.
    let rc = unsafe {
        sys::mpt_get_convert_back_proof(
            p.holder_privkey.as_bytes().as_ptr(),
            p.holder_pubkey.as_bytes().as_ptr(),
            p.context_hash.as_bytes().as_ptr(),
            p.amount,
            &params,
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ConvertBackProof::new(out))
}

// ─────────────────────────────────────────────────────────────────────────
//  Clawback: 64-byte compact sigma proof
// ─────────────────────────────────────────────────────────────────────────

/// Generates the 64-byte `ConfidentialMPTClawback` proof.
///
/// Issuer-only. Proves that the holder's `IssuerEncryptedBalance` ciphertext
/// (already on the ledger) decrypts to `amount` under the issuer's secret
/// key.
pub fn clawback(
    issuer_privkey: &Privkey,
    issuer_pubkey: &Pubkey,
    context_hash: &ContextHash,
    amount: u64,
    issuer_encrypted_balance: &Ciphertext,
) -> Result<ClawbackProof> {
    let mut out = [0u8; 64];
    // SAFETY: 32 / 33 / 32 / 66 / 64 buffer sizes match the FFI contract.
    let rc = unsafe {
        sys::mpt_get_clawback_proof(
            issuer_privkey.as_bytes().as_ptr(),
            issuer_pubkey.as_bytes().as_ptr(),
            context_hash.as_bytes().as_ptr(),
            amount,
            issuer_encrypted_balance.as_bytes().as_ptr(),
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(ClawbackProof::new(out))
}
