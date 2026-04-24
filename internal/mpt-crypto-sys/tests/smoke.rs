//! Runtime smoke test: exercises the full FFI chain — the dynamic linker
//! finds `libmpt-crypto.{dylib,so,dll}` (via rpath or PATH), symbols resolve,
//! and a C function call round-trips with sensible output.
//!
//! If this passes, the scaffold is wired correctly end-to-end.

use mpt_crypto_sys as sys;

#[test]
fn creates_and_destroys_a_secp256k1_context() {
    // SAFETY: secp256k1_context_create returns a valid pointer for valid
    //         flag combinations, or null on allocation failure. We pass
    //         SIGN|VERIFY, which is a valid combination documented in the
    //         upstream secp256k1.h.
    unsafe {
        let flags = sys::SECP256K1_CONTEXT_SIGN | sys::SECP256K1_CONTEXT_VERIFY;
        let ctx = sys::secp256k1_context_create(flags);
        assert!(!ctx.is_null(), "context_create returned null");
        sys::secp256k1_context_destroy(ctx);
    }
}

#[test]
fn generates_an_elgamal_keypair_with_nonzero_bytes() {
    // Validates that the upstream library was built with a working RNG
    // (OpenSSL statically linked inside the dylib).
    let mut privkey = [0u8; 32];
    let mut pubkey = [0u8; 33];

    // SAFETY: mpt_generate_keypair writes 32 bytes to privkey and 33 bytes
    //         to pubkey; buffer sizes match upstream's constants
    //         kMPT_PRIVKEY_SIZE and kMPT_PUBKEY_SIZE.
    let rc = unsafe {
        sys::mpt_generate_keypair(privkey.as_mut_ptr(), pubkey.as_mut_ptr())
    };
    assert_eq!(rc, 0, "mpt_generate_keypair returned {rc}");

    // Non-deterministic: all zeros would indicate the RNG did nothing.
    assert!(privkey.iter().any(|&b| b != 0), "privkey is all zeros");
    assert!(pubkey.iter().any(|&b| b != 0), "pubkey is all zeros");

    // Compressed secp256k1 pubkey starts with 0x02 or 0x03.
    assert!(
        pubkey[0] == 0x02 || pubkey[0] == 0x03,
        "pubkey prefix byte = 0x{:02x}, expected 0x02 or 0x03",
        pubkey[0]
    );
}

#[test]
fn compact_standard_proof_size_matches_spec() {
    // XLS-0096 §5.4: 192-byte compact AND-composed sigma proof for Send.
    assert_eq!(sys::SECP256K1_COMPACT_STANDARD_PROOF_SIZE, 192);
    assert_eq!(sys::SECP256K1_COMPACT_CONVERTBACK_PROOF_SIZE, 128);
    assert_eq!(sys::SECP256K1_COMPACT_CLAWBACK_PROOF_SIZE, 64);
    assert_eq!(sys::SECP256K1_POK_SK_PROOF_SIZE, 64);
}
