//! Shared validation helpers and constants for Multi-Purpose Token (XLS-33)
//! transactions.
//!
//! These rules are protocol-level invariants (XLS-33 / XLS-89), not specific to
//! any single transaction type. They are consumed by `MPTokenIssuanceCreate`,
//! `MPTokenIssuanceSet`, `MPTokenIssuanceDestroy`, `MPTokenAuthorize`, `Clawback`,
//! and the `MPTAmount` / `MPTCurrency` models, so they live in a neutral module
//! rather than in any one consumer.

use crate::core::addresscodec::decode_classic_address;
use crate::models::{XRPLModelException, XRPLModelResult};

/// Expected length (in hex characters) of an MPTokenIssuanceID:
/// 24 bytes (Hash192) = 48 hex chars.
const MPTOKEN_ISSUANCE_ID_HEX_LEN: usize = 48;

/// Validates that an `MPTokenIssuanceID` string is 48 ASCII hex characters
/// (24 bytes, Hash192 per XLS-33).
pub(crate) fn validate_mptoken_issuance_id(id: &str) -> XRPLModelResult<()> {
    if id.len() != MPTOKEN_ISSUANCE_ID_HEX_LEN || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "mptoken_issuance_id".into(),
            format: alloc::format!("{MPTOKEN_ISSUANCE_ID_HEX_LEN}-char ASCII hex string"),
            found: id.into(),
        });
    }
    Ok(())
}

/// Validates that a `holder` string decodes as a classic XRPL address.
pub(crate) fn validate_holder_address(holder: &str) -> XRPLModelResult<()> {
    if decode_classic_address(holder).is_err() {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "holder".into(),
            format: "classic XRPL address".into(),
            found: holder.into(),
        });
    }
    Ok(())
}

/// Expected length (in hex characters) of a DomainID (Hash256 = 32 bytes = 64 hex chars).
const DOMAIN_ID_HEX_LEN: usize = 64;

/// Validates that a `DomainID` is a 64-char ASCII hex string.
pub(crate) fn validate_domain_id(id: &str) -> XRPLModelResult<()> {
    if id.len() != DOMAIN_ID_HEX_LEN || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "domain_id".into(),
            format: alloc::format!("{DOMAIN_ID_HEX_LEN}-char ASCII hex string"),
            found: id.into(),
        });
    }
    Ok(())
}

/// Maximum transfer fee value (50000 = 50.000%). Shared by MPTokenIssuanceCreate
/// and MPTokenIssuanceSet.
pub(crate) const MAX_MPT_TRANSFER_FEE: u16 = 50000;

/// Validates that a transfer fee is within the allowed range (0–50000).
pub(crate) fn validate_transfer_fee(fee: u16) -> XRPLModelResult<()> {
    if fee > MAX_MPT_TRANSFER_FEE {
        return Err(XRPLModelException::ValueTooHigh {
            field: "transfer_fee".into(),
            max: MAX_MPT_TRANSFER_FEE as u32,
            found: fee as u32,
        });
    }
    Ok(())
}

/// Maximum MPT metadata byte length per XLS-89. Shared by MPTokenIssuanceCreate
/// and MPTokenIssuanceSet.
pub(crate) const MAX_MPT_METADATA_BYTES: usize = 1024;

/// Validates that MPT metadata is a non-empty, even-length, hex-encoded string ≤1024 bytes.
pub(crate) fn validate_mpt_metadata(metadata: &str) -> XRPLModelResult<()> {
    if metadata.is_empty()
        || !metadata.len().is_multiple_of(2)
        || !metadata.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "mptoken_metadata".into(),
            format: "non-empty even-length ASCII hex string".into(),
            found: metadata.into(),
        });
    }
    let byte_len = metadata.len() / 2;
    if byte_len > MAX_MPT_METADATA_BYTES {
        return Err(XRPLModelException::ValueTooLong {
            field: "mptoken_metadata".into(),
            max: MAX_MPT_METADATA_BYTES,
            found: byte_len,
        });
    }
    Ok(())
}
