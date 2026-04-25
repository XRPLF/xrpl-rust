//! EC-ElGamal encryption / decryption and blinding-factor generation.

use crate::{Error, Result, types::{BlindingFactor, Ciphertext, Privkey, Pubkey}};
use mpt_crypto_sys as sys;

/// Generates a fresh 32-byte blinding factor (the ElGamal randomness `r`).
///
/// Used both as the ElGamal randomness across multi-recipient ciphertexts
/// and (in Send proofs) as the Pedersen blinding factor for `AmountCommitment`
/// — see XLS-0096 §5.4 "reused randomness" optimization.
pub fn random_blinding_factor() -> Result<BlindingFactor> {
    let mut r = [0u8; 32];
    // SAFETY: `r` is exclusively borrowed; size matches the 32-byte contract.
    let rc = unsafe { sys::mpt_generate_blinding_factor(r.as_mut_ptr()) };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(BlindingFactor::new(r))
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

/// Decrypts a ciphertext using the holder's secret key, recovering the
/// original `u64` amount.
///
/// The C implementation uses a discrete-log lookup table for u64 — fast for
/// small / typical balances, slow if the recovered scalar is unusually large.
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
