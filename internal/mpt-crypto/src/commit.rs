//! Pedersen commitments on secp256k1.

use crate::{Error, Result, types::{BlindingFactor, Commitment}};
use mpt_crypto_sys as sys;

/// Computes a Pedersen commitment `PC = amount·G + blinding·H`.
///
/// `H` is a NUMS generator fixed by the upstream library; same `(amount,
/// blinding)` pair always produces the same commitment.
pub fn pedersen(amount: u64, blinding: &BlindingFactor) -> Result<Commitment> {
    let mut out = [0u8; 33];
    // SAFETY: 32-byte input + 33-byte output match the FFI contract.
    let rc = unsafe {
        sys::mpt_get_pedersen_commitment(
            amount,
            blinding.as_bytes().as_ptr(),
            out.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok(Commitment::new(out))
}
