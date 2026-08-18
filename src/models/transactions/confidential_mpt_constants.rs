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

use crate::core::addresscodec::decode_classic_address;
use crate::models::{XRPLModelException, XRPLModelResult};

/// Hex-character offsets of the issuer's `AccountID` within an
/// `MPTokenIssuanceID`. The 24-byte ID is `sequence(4) || issuerAccountID(20)`,
/// so the 20-byte issuer occupies hex characters `[8, 48)` (mirrors rippled's
/// `MPTIssue::getIssuer`, which skips the 4-byte sequence prefix).
const ISSUER_HEX_START: usize = 8;
const ISSUER_HEX_END: usize = 48;

/// Returns `true` when `address` (a classic XRPL address) is the issuer encoded
/// in `issuance_id_hex`.
///
/// Used to enforce rippled's `temMALFORMED` issuer-role preflight bans (e.g.
/// the issuer cannot be the sender/destination of a `ConfidentialMPTSend`, and
/// must be the account of a `ConfidentialMPTClawback`). A malformed address or
/// too-short issuance ID yields `false` — the dedicated format checks surface
/// those errors — so this only reports a *positive* issuer match.
pub(crate) fn address_is_issuer(issuance_id_hex: &str, address: &str) -> bool {
    if issuance_id_hex.len() < ISSUER_HEX_END {
        return false;
    }
    let Ok(account_id) = decode_classic_address(address) else {
        return false;
    };
    let issuer_hex = &issuance_id_hex[ISSUER_HEX_START..ISSUER_HEX_END];
    issuer_hex.eq_ignore_ascii_case(&hex::encode(account_id))
}

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

/// Maximum representable MPT amount: `2^63 - 1` (`i64::MAX`).
///
/// MPT amounts are serialized as an unsigned 64-bit integer but capped at
/// `i64::MAX` on-ledger, matching rippled's `maxMPTokenAmount` and the JS
/// SDK's `MAX_MPT_AMOUNT`. Values in `(i64::MAX, u64::MAX]` parse as `u64`
/// but are rejected here so the SDK fails fast instead of at the node.
pub(crate) const MAX_MPT_AMOUNT: u64 = i64::MAX as u64;

/// Validates the string-encoded `MPTAmount` field: an unsigned 64-bit
/// integer in `[0, 2^63 - 1]`, optionally required to be non-zero.
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
    if amount > MAX_MPT_AMOUNT {
        return Err(XRPLModelException::InvalidValueFormat {
            field: field.into(),
            format: alloc::format!("MPT amount in [0, {MAX_MPT_AMOUNT}] (2^63 - 1)"),
            found: value.into(),
        });
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
        // Upper bound: i64::MAX (2^63 - 1) is the last accepted value; the next
        // integer up still parses as u64 but exceeds the on-ledger MPT cap.
        assert!(validate_mpt_amount("mpt_amount", "9223372036854775807", false).is_ok());
        assert!(validate_mpt_amount("mpt_amount", "9223372036854775808", false).is_err());
        assert!(validate_mpt_amount("mpt_amount", "18446744073709551615", false).is_err());
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
