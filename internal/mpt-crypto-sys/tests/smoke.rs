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

// ─────────────────────────────────────────────────────────────────────────
//  Constant-documentation tests
//
//  These tests are the executable form of the spec-vs-implementation
//  agreement. They run during `cargo test -p mpt-crypto-sys` and fail
//  loudly if the linked libmpt-crypto ever drifts from what XLS-0096 says.
//  Treat the assertion strings as documentation: each one says where the
//  number comes from in the spec.
// ─────────────────────────────────────────────────────────────────────────

/// The four atomic sigma-proof sizes defined in `secp256k1_mpt.h`.
///
/// These are the irreducible primitives — every higher-level proof bundle
/// is a composition of these and the Bulletproof base sizes.
#[test]
fn sigma_proof_base_sizes() {
    // §7.2 / §A.7 — Schnorr Proof of Knowledge for holder-key registration.
    assert_eq!(sys::SECP256K1_POK_SK_PROOF_SIZE, 64);

    // §5.4 / §8.2 — compact AND-composed sigma in ConfidentialMPTSend.
    //   Witnesses: r, m, sk_A, rho, b  (5 scalars, encoded as Z_q^6 = 192 B).
    assert_eq!(sys::SECP256K1_COMPACT_STANDARD_PROOF_SIZE, 192);

    // §5.4 / §10.3 — compact sigma in ConfidentialMPTConvertBack.
    //   Witnesses: b, sk_A, rho  (3 scalars, encoded as Z_q^4 = 128 B).
    assert_eq!(sys::SECP256K1_COMPACT_CONVERTBACK_PROOF_SIZE, 128);

    // §5.4 / §11.2 — compact sigma in ConfidentialMPTClawback.
    //   Witnesses: sk_iss  (1 scalar, encoded as Z_q^2 = 64 B).
    assert_eq!(sys::SECP256K1_COMPACT_CLAWBACK_PROOF_SIZE, 64);
}

/// The three Bulletproof base sizes defined in `mpt_utility.h`.
///
/// Aggregated proves m=2 values in [0, 2^64) (used in Send: amount + remainder).
/// Single proves m=1 value in [0, 2^64) (used in ConvertBack: post-withdraw remainder).
#[test]
fn bulletproof_base_sizes() {
    // §5.4 / §10.3 — single 64-bit range proof (ConvertBack).
    assert_eq!(sys::kMPT_SINGLE_BULLETPROOF_SIZE, 688);

    // §5.4 / §8.2 / §14.1 — aggregated proof for two 64-bit values (Send).
    assert_eq!(sys::kMPT_DOUBLE_BULLETPROOF_SIZE, 754);
}

/// The utility layer's `kMPT_SCHNORR_PROOF_SIZE` aliases the primitive
/// layer's `SECP256K1_POK_SK_PROOF_SIZE`. Both identify the 64 B Schnorr PoK
/// used by ConfidentialMPTConvert.
///
/// If the two ever diverge, one of the layers has a stale definition and
/// the bindings will silently agree with the wrong one.
#[test]
fn schnorr_proof_aliases_are_consistent() {
    assert_eq!(sys::kMPT_SCHNORR_PROOF_SIZE, sys::SECP256K1_POK_SK_PROOF_SIZE);
    assert_eq!(sys::kMPT_SCHNORR_PROOF_SIZE, 64);
}

/// Composed proof sizes per XLS-0096 §5.4 / §14.1.
///
/// These are the totals the safe-wrapper `*Proof` newtypes ([u8; N]) hardcode.
/// Each total is `sigma_size + range_proof_size` (or just one of the two).
/// Asserting them by addition ties the wrappers to the base constants —
/// drift in either base value is caught here.
#[test]
fn total_proof_sizes_per_xls_0096() {
    // Convert: just the Schnorr PoK.  §7.2.
    assert_eq!(64, sys::SECP256K1_POK_SK_PROOF_SIZE);

    // Send: 192 (compact sigma) + 754 (aggregated Bulletproof) = 946.  §8.2.
    let send_total =
        sys::SECP256K1_COMPACT_STANDARD_PROOF_SIZE + sys::kMPT_DOUBLE_BULLETPROOF_SIZE;
    assert_eq!(send_total, 946,
        "Send proof composes to 192 + 754 = 946 per §5.4 / §14.1");

    // ConvertBack: 128 (compact sigma) + 688 (single Bulletproof) = 816.  §10.3.
    let convert_back_total =
        sys::SECP256K1_COMPACT_CONVERTBACK_PROOF_SIZE + sys::kMPT_SINGLE_BULLETPROOF_SIZE;
    assert_eq!(convert_back_total, 816,
        "ConvertBack proof composes to 128 + 688 = 816 per §5.4 / §10.3");

    // Clawback: just the compact sigma.  §11.2.
    assert_eq!(64, sys::SECP256K1_COMPACT_CLAWBACK_PROOF_SIZE);

    // MergeInbox is intentionally absent — proof-free per §9 / §A.2.
}

/// Wire sizes for cryptographic blobs — the field types every confidential
/// transaction carries.
///
/// These match the XLS-0096 transaction-field tables (§7.2, §8.2, §10.3,
/// §11.2): every `Blob` field documented as N bytes corresponds to one of
/// these constants.
#[test]
fn wire_size_primitives() {
    // Compressed secp256k1 public key = 33 bytes (1 prefix byte + 32-byte X).
    // Used by HolderEncryptionKey / IssuerEncryptionKey / AuditorEncryptionKey.
    assert_eq!(sys::kMPT_PUBKEY_SIZE, 33);

    // 32-byte secret scalar — Privkey, BlindingFactor, ContextHash all share.
    assert_eq!(sys::kMPT_PRIVKEY_SIZE, 32);
    assert_eq!(sys::kMPT_BLINDING_FACTOR_SIZE, 32);
    assert_eq!(sys::kMPT_HALF_SHA_SIZE, 32);     // SHA-256 / 2 (truncated 32 B output)

    // ElGamal ciphertext = (R, S), each a compressed point.
    //   R alone = 33 B; (R, S) total = 66 B.
    // Used as HolderEncryptedAmount / IssuerEncryptedAmount / AuditorEncryptedAmount,
    // and as on-ledger CB_S, CB_IN, IssuerEncryptedBalance, AuditorEncryptedBalance.
    assert_eq!(sys::kMPT_ELGAMAL_CIPHER_SIZE, 33);
    assert_eq!(sys::kMPT_ELGAMAL_TOTAL_SIZE, 66);
    assert_eq!(
        sys::kMPT_ELGAMAL_TOTAL_SIZE,
        sys::kMPT_ELGAMAL_CIPHER_SIZE * 2,
        "ElGamal ciphertext is exactly two compressed points"
    );

    // Pedersen commitment = single compressed point.
    // Used for AmountCommitment and BalanceCommitment.
    assert_eq!(sys::kMPT_PEDERSEN_COMMIT_SIZE, 33);
}

/// XRPL ledger-identifier sizes used to build per-transaction context hashes.
#[test]
fn ledger_identifier_sizes() {
    // 20-byte XRPL classic-address payload (RIPEMD-160 of a SHA-256).
    assert_eq!(sys::kMPT_ACCOUNT_ID_SIZE, 20);

    // 24-byte MPTokenIssuanceID = 4 byte sequence || 20 byte issuer AccountID.
    // Defined by XLS-33; XLS-0096 reuses verbatim.
    assert_eq!(sys::kMPT_ISSUANCE_ID_SIZE, 24);
}

/// Transaction-type IDs assigned to confidential MPT transactions.
///
/// These numbers are the contract between the C library, rippled's
/// transaction-dispatch table, and any client serializing `TransactionType`
/// to wire format. If they ever drift, every binary-codec round-trip breaks.
#[test]
fn transaction_type_ids_match_xls_0096() {
    // §4.1 of the spec assigns these directly.
    assert_eq!(sys::ttCONFIDENTIAL_MPT_CONVERT,      85);
    assert_eq!(sys::ttCONFIDENTIAL_MPT_MERGE_INBOX,  86);
    assert_eq!(sys::ttCONFIDENTIAL_MPT_CONVERT_BACK, 87);
    assert_eq!(sys::ttCONFIDENTIAL_MPT_SEND,         88);
    assert_eq!(sys::ttCONFIDENTIAL_MPT_CLAWBACK,     89);
}
