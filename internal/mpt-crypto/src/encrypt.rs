//! EC-ElGamal encryption / decryption and blinding-factor generation.

use zeroize::Zeroizing;

use crate::{
    Error, Result,
    types::{BlindingFactor, Ciphertext, Privkey, Pubkey},
};
use mpt_crypto_sys as sys;

/// Generates a fresh 32-byte blinding factor (the ElGamal randomness `r`).
///
/// Used both as the ElGamal randomness across multi-recipient ciphertexts
/// and (in Send proofs) as the Pedersen blinding factor for `AmountCommitment`
/// — see XLS-0096 §5.4 "reused randomness" optimization.
pub fn random_blinding_factor() -> Result<BlindingFactor> {
    // Zeroize the intermediate secret scalar on drop, so it is not left in a
    // freed stack frame after being copied into the BlindingFactor wrapper.
    let mut r = Zeroizing::new([0u8; 32]);
    // SAFETY: `r` is exclusively borrowed; size matches the 32-byte contract.
    let rc = unsafe { sys::mpt_generate_blinding_factor(r.as_mut_ptr()) };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(BlindingFactor::new(*r))
}

/// Encrypts a 64-bit `amount` under `pubkey` with the supplied `blinding`.
///
/// Result is the 66-byte ciphertext `(R = r·G, S = m·G + r·pk)`.
/// Reusing the same blinding across multiple ciphertexts under different keys
/// produces "shared-r" ciphertexts that the compact sigma proof relies on.
pub fn encrypt(amount: u64, pubkey: &Pubkey, blinding: &BlindingFactor) -> Result<Ciphertext> {
    let mut out = [0u8; 66];
    // SAFETY: pointers reference fixed-size arrays whose lengths match the
    //         FFI contract (33, 32, 66 bytes).
    let rc = unsafe {
        sys::mpt_encrypt_amount(
            amount,
            pubkey.as_bytes().as_ptr(),
            blinding.as_bytes().as_ptr(),
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(Ciphertext::new(out))
}

/// Search width, in bits, that [`decrypt`] covers: amounts in `[0, 2^40)`.
///
/// Matches upstream's `MPT_BSGS_DEFAULT_BITS_TOTAL`. Use [`decrypt_with_bits`]
/// for wider (or narrower) widths.
pub const DEFAULT_DECRYPT_BITS: u32 = 40;

/// Largest search width accepted by [`decrypt_with_bits`]; MPT amounts never
/// exceed `2^63 - 1`.
pub const MAX_DECRYPT_BITS: u32 = 63;

/// Decrypts a ciphertext using the holder's secret key, recovering amounts in
/// `[0, 2^DEFAULT_DECRYPT_BITS)`.
///
/// Uses upstream's Baby-Step Giant-Step solver. The first call builds a ~22 MB
/// lookup table (several seconds) that later calls reuse for the life of the
/// process. Decryption is **not constant-time**: it takes longer for larger
/// amounts, so do not expose its latency to untrusted observers.
pub fn decrypt(ciphertext: &Ciphertext, privkey: &Privkey) -> Result<u64> {
    let mut amount: u64 = 0;
    // SAFETY: pointers are valid for the call; `&mut amount` is exclusive.
    let rc = unsafe {
        sys::mpt_decrypt_amount(
            ciphertext.as_bytes().as_ptr(),
            privkey.as_bytes().as_ptr(),
            &mut amount,
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(amount)
}

/// Decrypts a ciphertext known to encrypt an amount below `2^bits`.
///
/// `bits` must be in `1..=MAX_DECRYPT_BITS`; otherwise the underlying call
/// returns `-2`, surfaced here as [`Error::NonZeroRc`]. [`bits_for`] derives it
/// from a known upper bound (e.g. the issuance's `ConfidentialOutstandingAmount`).
/// Each width builds its own lookup table on first use, and wider widths take
/// longer to build and to search; a failed decrypt (wrong key, or an amount of
/// `2^bits` or more) searches the whole width before returning. Not
/// constant-time; see [`decrypt`].
pub fn decrypt_with_bits(ciphertext: &Ciphertext, privkey: &Privkey, bits: u32) -> Result<u64> {
    // Out-of-range widths wrap to -1 here; the C side rejects them with -2.
    let bits = i32::try_from(bits).unwrap_or(-1);
    let mut amount: u64 = 0;
    // SAFETY: pointers are valid for the call; `&mut amount` is exclusive.
    let rc = unsafe {
        sys::mpt_decrypt_amount_with_bits_total(
            ciphertext.as_bytes().as_ptr(),
            privkey.as_bytes().as_ptr(),
            bits,
            &mut amount,
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(amount)
}

/// Like [`decrypt_with_bits`], but the caller also picks `l1`, the baby-step
/// width: the lookup table holds `2^(l1 - 1)` entries, so each +1 doubles its
/// memory and build time and halves the search steps.
///
/// `l1` must be in `1..=32` and below `bits`; otherwise the underlying call
/// returns `-2`. Prefer [`decrypt_with_bits`] unless you need to trade memory
/// for speed.
pub fn decrypt_with_bits_and_l1(
    ciphertext: &Ciphertext,
    privkey: &Privkey,
    bits: u32,
    l1: u32,
) -> Result<u64> {
    // Out-of-range values wrap to -1 here; the C side rejects them with -2.
    let bits = i32::try_from(bits).unwrap_or(-1);
    let l1 = i32::try_from(l1).unwrap_or(-1);
    let mut amount: u64 = 0;
    // SAFETY: pointers are valid for the call; `&mut amount` is exclusive.
    let rc = unsafe {
        sys::mpt_decrypt_amount_with_l1_bits_total(
            ciphertext.as_bytes().as_ptr(),
            privkey.as_bytes().as_ptr(),
            bits,
            l1,
            &mut amount,
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(amount)
}

/// Smallest search width whose range `[0, 2^bits)` contains `max_amount`.
pub fn bits_for(max_amount: u64) -> u32 {
    (u64::BITS - max_amount.leading_zeros()).max(1)
}
