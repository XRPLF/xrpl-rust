//! Homomorphic operations on EC-ElGamal ciphertexts.
//!
//! ElGamal is additively homomorphic: `Enc(a) ± Enc(b) = Enc(a ± b)` when both
//! ciphertexts are encrypted under the same public key. These wrap mpt-crypto's
//! `secp256k1_elgamal_add` / `secp256k1_elgamal_subtract`.
//!
//! They are the building block for **predicting a confidential balance's next
//! state client-side** — e.g. the `new CB_S = CB_S ⊖ SenderEncryptedAmount`
//! update rippled applies to a sender's `ConfidentialBalanceSpending` on a send
//! (see rippled's `chainAfterSend` / `homomorphicSubtract`), so a client can
//! compute the balance a subsequent proof must bind to without decrypting.

use crate::{Error, Result, types::Ciphertext};
use mpt_crypto_sys as sys;

/// Parse one 33-byte compressed secp256k1 point into libsecp256k1's internal
/// (opaque, 64-byte) representation, which the elgamal ops operate on.
fn parse_point(ctx: *const sys::secp256k1_context, bytes: &[u8]) -> Result<sys::secp256k1_pubkey> {
    let mut point = sys::secp256k1_pubkey { data: [0u8; 64] };
    // SAFETY: `ctx` is valid; `bytes` is a 33-byte compressed point; `point` is
    //         exclusively borrowed.
    let rc =
        unsafe { sys::secp256k1_ec_pubkey_parse(ctx, &mut point, bytes.as_ptr(), bytes.len()) };
    if rc != 1 {
        return Err(Error::Invariant("failed to parse ElGamal ciphertext point"));
    }
    Ok(point)
}

/// Serialize an internal point back to its 33-byte compressed encoding.
fn serialize_point(
    ctx: *const sys::secp256k1_context,
    point: &sys::secp256k1_pubkey,
) -> Result<[u8; 33]> {
    let mut out = [0u8; 33];
    let mut out_len: usize = out.len();
    // SAFETY: `out` has 33 bytes of capacity; `point` is a valid parsed key;
    //         COMPRESSED serialization yields exactly 33 bytes.
    let rc = unsafe {
        sys::secp256k1_ec_pubkey_serialize(
            ctx,
            out.as_mut_ptr(),
            &mut out_len,
            point,
            sys::SECP256K1_EC_COMPRESSED,
        )
    };
    if rc != 1 || out_len != 33 {
        return Err(Error::Invariant(
            "failed to serialize ElGamal ciphertext point",
        ));
    }
    Ok(out)
}

/// Homomorphically combine two ciphertexts under the same key. A `Ciphertext`
/// is two compressed points `R || S` (33 bytes each), so we parse the four
/// input points, call the elgamal op, and re-serialize the two output points.
fn combine(a: &Ciphertext, b: &Ciphertext, subtract: bool) -> Result<Ciphertext> {
    // SAFETY: `mpt_secp256k1_context` returns libmpt-crypto's shared, valid
    //         context (owned by the library; never destroyed here).
    let ctx = unsafe { sys::mpt_secp256k1_context() };
    if ctx.is_null() {
        return Err(Error::Invariant("secp256k1 context unavailable"));
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let a_c1 = parse_point(ctx, &a_bytes[0..33])?;
    let a_c2 = parse_point(ctx, &a_bytes[33..66])?;
    let b_c1 = parse_point(ctx, &b_bytes[0..33])?;
    let b_c2 = parse_point(ctx, &b_bytes[33..66])?;

    let mut out_c1 = sys::secp256k1_pubkey { data: [0u8; 64] };
    let mut out_c2 = sys::secp256k1_pubkey { data: [0u8; 64] };

    // SAFETY: all four inputs are valid parsed points; both outputs are
    //         exclusively borrowed and 64 bytes as the FFI contract requires.
    let rc = unsafe {
        if subtract {
            sys::secp256k1_elgamal_subtract(
                ctx,
                &mut out_c1,
                &mut out_c2,
                &a_c1,
                &a_c2,
                &b_c1,
                &b_c2,
            )
        } else {
            sys::secp256k1_elgamal_add(ctx, &mut out_c1, &mut out_c2, &a_c1, &a_c2, &b_c1, &b_c2)
        }
    };
    // The secp256k1_elgamal_{add,subtract} entry points follow libsecp256k1's
    // house convention (`1` = success, `0` = failure), matching every other
    // `secp256k1_*` call in this crate; treat anything but `1` as failure so a
    // convention change surfaces as an error rather than a bad ciphertext.
    if rc != 1 {
        return Err(Error::Invariant("homomorphic ciphertext operation failed"));
    }

    let mut out = [0u8; 66];
    out[0..33].copy_from_slice(&serialize_point(ctx, &out_c1)?);
    out[33..66].copy_from_slice(&serialize_point(ctx, &out_c2)?);
    Ok(Ciphertext::new(out))
}

/// Homomorphically adds two ElGamal ciphertexts: `Enc(a) + Enc(b) = Enc(a + b)`
/// (both encrypted under the same public key).
///
/// Predicts a balance credited by an inbound amount (e.g. a merged inbox).
pub fn add_ciphertexts(a: &Ciphertext, b: &Ciphertext) -> Result<Ciphertext> {
    combine(a, b, false)
}

/// Homomorphically subtracts two ElGamal ciphertexts: `Enc(a) - Enc(b) =
/// Enc(a - b)` (both encrypted under the same public key).
///
/// This is the rule rippled applies to a sender's `ConfidentialBalanceSpending`
/// on a send — `new CB_S = CB_S ⊖ SenderEncryptedAmount` (see `chainAfterSend`).
/// Use it to predict the next spending balance client-side.
pub fn subtract_ciphertexts(a: &Ciphertext, b: &Ciphertext) -> Result<Ciphertext> {
    combine(a, b, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encrypt, keypair};

    #[test]
    fn homomorphic_add_and_subtract_roundtrip() {
        let (sk, pk) = keypair::generate().unwrap();
        let a = encrypt::encrypt(70, &pk, &encrypt::random_blinding_factor().unwrap()).unwrap();
        let b = encrypt::encrypt(30, &pk, &encrypt::random_blinding_factor().unwrap()).unwrap();

        // Enc(70) + Enc(30) decrypts to 100.
        let sum = add_ciphertexts(&a, &b).unwrap();
        assert_eq!(encrypt::decrypt(&sum, &sk).unwrap(), 100);

        // Enc(70) - Enc(30) decrypts to 40 — the CB_S-after-send update rule.
        let diff = subtract_ciphertexts(&a, &b).unwrap();
        assert_eq!(encrypt::decrypt(&diff, &sk).unwrap(), 40);
    }
}
