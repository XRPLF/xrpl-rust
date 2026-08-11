//! ElGamal/secp256k1 keypair generation.

use zeroize::Zeroizing;

use crate::{
    Error, Result,
    types::{Privkey, Pubkey},
};
use mpt_crypto_sys as sys;

/// Generates a fresh ElGamal keypair.
///
/// Internally calls into libmpt-crypto, which uses OpenSSL's RNG (statically
/// linked into the dylib). The private key is wiped from memory when the
/// returned `Privkey` is dropped.
pub fn generate() -> Result<(Privkey, Pubkey)> {
    // Zeroize the intermediate secret buffer on drop, so the private key is not
    // left in a freed stack frame after being copied into the Privkey wrapper.
    let mut sk = Zeroizing::new([0u8; 32]);
    let mut pk = [0u8; 33]; // public key — not secret

    // SAFETY: `sk` and `pk` are mutable for the duration of the call;
    //         their sizes match the FFI contract (32 / 33 bytes).
    let rc = unsafe { sys::mpt_generate_keypair(sk.as_mut_ptr(), pk.as_mut_ptr()) };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok((Privkey::new(*sk), Pubkey::new(pk)))
}

/// Derives an ElGamal keypair from a caller-supplied 32-byte secret scalar.
///
/// Unlike [`generate`] (fresh randomness), this lets the holder key be
/// **recovered from a seed**: derive a 32-byte scalar from a wallet seed with
/// your own KDF (e.g. a `ripple-keypairs`-style derivation), pass it here, and
/// get back the matching compressed ElGamal public key — so the key need not be
/// persisted out of band. The scalar must be a valid secp256k1 private key
/// (non-zero and below the curve order); otherwise an error is returned.
///
/// The secret is wiped from memory when the returned [`Privkey`] is dropped.
pub fn from_secret_key(secret: [u8; 32]) -> Result<(Privkey, Pubkey)> {
    // `secret` is a `Copy` value, so this function owns its own copy. Wrap it so
    // that copy is zeroized on *every* exit path (including the error paths
    // below), rather than lingering in a freed stack frame.
    let secret = Zeroizing::new(secret);

    // SAFETY: `mpt_secp256k1_context` returns libmpt-crypto's shared, valid,
    //         thread-safe context (owned by the library; never destroyed here).
    let ctx = unsafe { sys::mpt_secp256k1_context() };
    if ctx.is_null() {
        return Err(Error::Invariant("secp256k1 context unavailable"));
    }

    // SAFETY: `secret` is 32 bytes as the FFI contract requires; `ctx` is valid.
    if unsafe { sys::secp256k1_ec_seckey_verify(ctx, secret.as_ptr()) } != 1 {
        return Err(Error::Invariant(
            "secret is not a valid secp256k1 private key",
        ));
    }

    let mut pubkey = sys::secp256k1_pubkey { data: [0u8; 64] };
    // SAFETY: `ctx` valid, `pubkey` exclusively borrowed, `secret` is 32 bytes.
    if unsafe { sys::secp256k1_ec_pubkey_create(ctx, &mut pubkey, secret.as_ptr()) } != 1 {
        return Err(Error::Invariant("failed to derive public key from secret"));
    }

    let mut out = [0u8; 33];
    let mut out_len: usize = out.len();
    // SAFETY: `out` has `out_len` capacity; `pubkey` is a valid parsed key;
    //         COMPRESSED serialization yields exactly 33 bytes.
    let rc = unsafe {
        sys::secp256k1_ec_pubkey_serialize(
            ctx,
            out.as_mut_ptr(),
            &mut out_len,
            &pubkey,
            sys::SECP256K1_EC_COMPRESSED,
        )
    };
    if rc != 1 || out_len != 33 {
        return Err(Error::Invariant(
            "failed to serialize compressed public key",
        ));
    }

    Ok((Privkey::new(*secret), Pubkey::new(out)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_secret_key_matches_generate() {
        // A key derived from an existing private scalar must reproduce the same
        // public key — i.e. derivation is deterministic and recoverable.
        let (sk, pk) = generate().unwrap();
        let (sk2, pk2) = from_secret_key(*sk.as_bytes()).unwrap();
        assert_eq!(sk.as_bytes(), sk2.as_bytes());
        assert_eq!(pk.as_bytes(), pk2.as_bytes());
    }

    #[test]
    fn from_secret_key_rejects_invalid_scalar() {
        // The all-zero scalar is not a valid secp256k1 private key.
        assert!(from_secret_key([0u8; 32]).is_err());
    }
}
