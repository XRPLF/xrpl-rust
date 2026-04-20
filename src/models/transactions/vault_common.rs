//! Shared validation helpers for XLS-65 Vault transactions.

use alloc::string::ToString;

use crate::models::{XRPLModelException, XRPLModelResult};

/// The canonical length, in hex characters, of a VaultID.
///
/// A VaultID is a 256-bit hash, which is 32 bytes or 64 hex characters
/// when serialized on the wire.
pub(crate) const VAULT_ID_HEX_LEN: usize = 64;

/// Validate a VaultID value: must be exactly 64 ASCII hex characters.
///
/// Used by every vault transaction that references an existing vault
/// (VaultSet, VaultDelete, VaultDeposit, VaultWithdraw, VaultClawback).
pub(crate) fn validate_vault_id(vault_id: &str) -> XRPLModelResult<()> {
    if vault_id.len() != VAULT_ID_HEX_LEN {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "vault_id".to_string(),
            format: "64 hex characters (256-bit hash)".to_string(),
            found: vault_id.to_string(),
        });
    }
    if !vault_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: "vault_id".to_string(),
            format: "ASCII hexadecimal".to_string(),
            found: vault_id.to_string(),
        });
    }
    Ok(())
}

/// Validate a hex-encoded blob field: must be pure ASCII hex and not exceed
/// `max_hex_chars` characters in length (2 hex chars per byte).
pub(crate) fn validate_hex_blob(
    field: &'static str,
    value: &str,
    max_hex_chars: usize,
) -> XRPLModelResult<()> {
    if value.len() > max_hex_chars {
        return Err(XRPLModelException::ValueTooLong {
            field: field.to_string(),
            max: max_hex_chars,
            found: value.len(),
        });
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValueFormat {
            field: field.to_string(),
            format: "ASCII hexadecimal".to_string(),
            found: value.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_vault_id_accepts_valid_id() {
        let id = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";
        assert!(validate_vault_id(id).is_ok());
    }

    #[test]
    fn test_validate_vault_id_rejects_wrong_length() {
        assert!(validate_vault_id("DEADBEEF").is_err());
        let too_long = "A".repeat(65);
        assert!(validate_vault_id(&too_long).is_err());
    }

    #[test]
    fn test_validate_vault_id_rejects_non_hex() {
        let id = "Z0000000000000000000000000000000000000000000000000000000DEADBEEF";
        assert!(validate_vault_id(id).is_err());
    }

    #[test]
    fn test_validate_hex_blob_accepts_valid() {
        assert!(validate_hex_blob("data", "48656C6C6F", 512).is_ok());
        assert!(validate_hex_blob("data", "", 512).is_ok());
    }

    #[test]
    fn test_validate_hex_blob_rejects_too_long() {
        let long = "A".repeat(513);
        assert!(validate_hex_blob("data", &long, 512).is_err());
    }

    #[test]
    fn test_validate_hex_blob_rejects_non_hex() {
        assert!(validate_hex_blob("data", "XYZ", 512).is_err());
    }
}
