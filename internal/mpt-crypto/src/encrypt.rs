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

/// Default upper bound for the discrete-log search performed by [`decrypt`].
///
/// `mpt_decrypt_amount` (mpt-crypto >= 1.0) recovers the amount by searching the
/// inclusive range `[range_low, range_high]`; cost scales linearly with its
/// width (~3s for `0..=1_000_000` on Apple Silicon). Matches xrpl-py's
/// `DEFAULT_DECRYPT_RANGE_HIGH`. Use [`decrypt_in_range`] for other windows.
pub const DEFAULT_DECRYPT_RANGE_HIGH: u64 = 1_000_000;

/// Decrypts a ciphertext using the holder's secret key, recovering the original
/// `u64` amount by searching `0..=DEFAULT_DECRYPT_RANGE_HIGH`.
///
/// The C implementation uses a discrete-log lookup table for u64 — fast for
/// small / typical balances, slow over wide ranges. Use [`decrypt_in_range`] to
/// search a different window.
pub fn decrypt(ciphertext: &Ciphertext, privkey: &Privkey) -> Result<u64> {
    decrypt_in_range(ciphertext, privkey, 0, DEFAULT_DECRYPT_RANGE_HIGH)
}

/// Decrypts a ciphertext, searching the inclusive range `[range_low, range_high]`.
///
/// `range_high` must be `< u64::MAX` and `>= range_low`; otherwise the underlying
/// call returns `-2`, surfaced here as [`Error::NonZeroRc`].
pub fn decrypt_in_range(
    ciphertext: &Ciphertext,
    privkey: &Privkey,
    range_low: u64,
    range_high: u64,
) -> Result<u64> {
    let mut amount: u64 = 0;
    // SAFETY: pointers are valid for the call; `&mut amount` is exclusive.
    let rc = unsafe {
        sys::mpt_decrypt_amount(
            ciphertext.as_bytes().as_ptr(),
            privkey.as_bytes().as_ptr(),
            &mut amount,
            range_low,
            range_high,
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(amount)
}
