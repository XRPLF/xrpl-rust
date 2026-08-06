//! Canonical sizes for Confidential MPT (XLS-0096) fields and zero-knowledge
//! proofs.
//!
//! Single source of truth for the model-layer length validations shared by the
//! `ConfidentialMPT*` transactions. The byte sizes mirror the mpt-crypto
//! library constants (`SECP256K1_*_PROOF_SIZE` / `kMPT_*`); the
//! `internal/mpt-crypto` integration tests cross-check them against the
//! compiled library so the two cannot silently drift across a version bump.
//!
//! This module carries no dependency on the native `mpt-crypto` bindings, so
//! the pure transaction models validate without the crypto feature enabled.

// --- Primitive byte sizes (mirror mpt_protocol.h) ---

/// Schnorr proof of knowledge of an ElGamal secret key.
pub const SCHNORR_PROOF_SIZE: usize = 64;
/// EC-ElGamal ciphertext (two compressed points).
pub const ELGAMAL_TOTAL_SIZE: usize = 66;
/// Pedersen commitment (one compressed point).
pub const PEDERSEN_COMMIT_SIZE: usize = 33;
/// ElGamal randomness / Pedersen blinding scalar.
pub const BLINDING_FACTOR_SIZE: usize = 32;
/// Compressed secp256k1 public key.
pub const PUBKEY_COMPRESSED_SIZE: usize = 33;

// --- Compact sigma + bulletproof component byte sizes ---

/// Compact sigma proof carried by `ConfidentialMPTClawback`.
pub const COMPACT_CLAWBACK_PROOF_SIZE: usize = 64;
/// Compact sigma proof carried by `ConfidentialMPTConvertBack`.
pub const COMPACT_CONVERTBACK_PROOF_SIZE: usize = 128;
/// Compact sigma proof carried by `ConfidentialMPTSend`.
pub const COMPACT_STANDARD_PROOF_SIZE: usize = 192;
/// Bulletproof over a single value (remaining balance).
pub const SINGLE_BULLETPROOF_SIZE: usize = 688;
/// Aggregated Bulletproof over two values (amount + remainder).
pub const DOUBLE_BULLETPROOF_SIZE: usize = 754;

// --- Composite `ZKProof` byte sizes ---

/// Total `ZKProof` size for `ConfidentialMPTClawback` (64 B).
pub const CLAWBACK_PROOF_SIZE: usize = COMPACT_CLAWBACK_PROOF_SIZE;
/// Total `ZKProof` size for `ConfidentialMPTConvertBack` (816 B).
pub const CONVERT_BACK_PROOF_SIZE: usize = COMPACT_CONVERTBACK_PROOF_SIZE + SINGLE_BULLETPROOF_SIZE;
/// Total `ZKProof` size for `ConfidentialMPTSend` (946 B).
pub const SEND_PROOF_SIZE: usize = COMPACT_STANDARD_PROOF_SIZE + DOUBLE_BULLETPROOF_SIZE;

// --- Hex-character lengths used by model validation (2 chars per byte) ---

/// `HolderEncryptionKey` / `IssuerEncryptionKey` / `AuditorEncryptionKey` (66).
pub const ENCRYPTION_KEY_LENGTH: usize = PUBKEY_COMPRESSED_SIZE * 2;
/// `BlindingFactor` (64).
pub const BLINDING_FACTOR_LENGTH: usize = BLINDING_FACTOR_SIZE * 2;
/// Schnorr `ZKProof` on a registering `ConfidentialMPTConvert` (128).
pub const SCHNORR_PROOF_LENGTH: usize = SCHNORR_PROOF_SIZE * 2;
/// Any `*EncryptedAmount` ciphertext field (132).
pub const CIPHERTEXT_LENGTH: usize = ELGAMAL_TOTAL_SIZE * 2;
/// `AmountCommitment` / `BalanceCommitment` (66).
pub const COMMITMENT_LENGTH: usize = PEDERSEN_COMMIT_SIZE * 2;
/// `ConfidentialMPTClawback::zk_proof` (128).
pub const CLAWBACK_PROOF_LENGTH: usize = CLAWBACK_PROOF_SIZE * 2;
/// `ConfidentialMPTSend::zk_proof` (1892).
pub const SEND_PROOF_LENGTH: usize = SEND_PROOF_SIZE * 2;
/// `ConfidentialMPTConvertBack::zk_proof` (1632).
pub const CONVERT_BACK_PROOF_LENGTH: usize = CONVERT_BACK_PROOF_SIZE * 2;

use crate::models::{XRPLModelException, XRPLModelResult};

/// Validates that `value` is exactly `expected` ASCII hex characters.
///
/// Shared by the `ConfidentialMPT*` models and by `MPTokenIssuanceSet`'s
/// encryption-key fields, mirroring rippled's `temBAD_CIPHERTEXT` /
/// `temMALFORMED` preflight checks.
pub(crate) fn validate_hex_length(
    field: &str,
    value: &str,
    expected: usize,
) -> XRPLModelResult<()> {
    if value.len() != expected || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: field.into(),
            format: alloc::format!("{expected}-char ASCII hex string ({} bytes)", expected / 2),
            found: alloc::format!("{}-char value", value.len()),
        });
    }
    Ok(())
}

/// Validates the string-encoded `MPTAmount` field: an unsigned 64-bit
/// integer, optionally required to be non-zero.
pub(crate) fn validate_mpt_amount(
    field: &str,
    value: &str,
    must_be_positive: bool,
) -> XRPLModelResult<()> {
    let amount: u64 = value
        .parse()
        .map_err(|_| XRPLModelException::InvalidValueFormat {
            field: field.into(),
            format: "string-encoded unsigned 64-bit integer".into(),
            found: value.into(),
        })?;
    if must_be_positive && amount == 0 {
        return Err(XRPLModelException::ValueZero(field.into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mpt_amount_validation() {
        assert!(validate_mpt_amount("mpt_amount", "0", false).is_ok());
        assert!(validate_mpt_amount("mpt_amount", "0", true).is_err());
        assert!(validate_mpt_amount("mpt_amount", "1000", true).is_ok());
        assert!(validate_mpt_amount("mpt_amount", "-1", false).is_err());
        assert!(validate_mpt_amount("mpt_amount", "abc", false).is_err());
    }

    #[test]
    fn composite_proof_sizes_match_xls_0096() {
        assert_eq!(CLAWBACK_PROOF_SIZE, 64);
        assert_eq!(CONVERT_BACK_PROOF_SIZE, 816);
        assert_eq!(SEND_PROOF_SIZE, 946);
        assert_eq!(CLAWBACK_PROOF_LENGTH, 128);
        assert_eq!(CONVERT_BACK_PROOF_LENGTH, 1632);
        assert_eq!(SEND_PROOF_LENGTH, 1892);
        assert_eq!(CIPHERTEXT_LENGTH, 132);
        assert_eq!(COMMITMENT_LENGTH, 66);
        assert_eq!(ENCRYPTION_KEY_LENGTH, 66);
        assert_eq!(BLINDING_FACTOR_LENGTH, 64);
        assert_eq!(SCHNORR_PROOF_LENGTH, 128);
    }

    #[test]
    fn hex_length_validation_rejects_wrong_length_and_non_hex() {
        assert!(validate_hex_length("zk_proof", &"AB".repeat(64), 128).is_ok());
        assert!(validate_hex_length("zk_proof", &"AB".repeat(63), 128).is_err());
        // Right length, but not hex.
        assert!(validate_hex_length("zk_proof", &"ZZ".repeat(64), 128).is_err());
    }
}
