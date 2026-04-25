//! ElGamal/secp256k1 keypair generation.

use crate::{Error, Result, types::{Privkey, Pubkey}};
use mpt_crypto_sys as sys;

/// Generates a fresh ElGamal keypair.
///
/// Internally calls into libmpt-crypto, which uses OpenSSL's RNG (statically
/// linked into the dylib). The private key is wiped from memory when the
/// returned `Privkey` is dropped.
pub fn generate() -> Result<(Privkey, Pubkey)> {
    let mut sk = [0u8; 32];
    let mut pk = [0u8; 33];

    // SAFETY: `sk` and `pk` are mutable for the duration of the call;
    //         their sizes match the FFI contract (32 / 33 bytes).
    let rc = unsafe { sys::mpt_generate_keypair(sk.as_mut_ptr(), pk.as_mut_ptr()) };
    if rc != 0 {
        return Err(Error::NonZeroRc(rc));
    }
    Ok((Privkey::new(sk), Pubkey::new(pk)))
}
