use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::_serde::opt_lgr_obj_flags;
use crate::core::addresscodec::decode_classic_address;
use crate::models::{
    ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag,
    transactions::{Transaction, TransactionType},
    FlagCollection, Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};
use crate::models::transactions::mptoken_issuance_create::{
    TIF_MPTOKENISSUANCE_IMMUTABLE_MASK, TIF_MPTOKENISSUANCE_VALID_MASK,
};

use super::{CommonFields, CommonTransactionBuilder};

/// Expected length (in hex characters) of an MPTokenIssuanceID:
/// 24 bytes (Hash192) = 48 hex chars.
const MPTOKEN_ISSUANCE_ID_HEX_LEN: usize = 48;

/// Transactions of the MPTokenIssuanceSet type support additional values
/// in the Flags field.
///
/// See MPTokenIssuanceSet flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenIssuanceSetFlag {
    /// Lock the MPT at the issuance or individual holder level.
    TfMPTLock = 0x00000001,
    /// Unlock the MPT at the issuance or individual holder level.
    TfMPTUnlock = 0x00000002,
    /// Enable the `lsfMPTCanLock` capability flag (one-way; cannot be unset).
    TfMPTSetCanLock = 0x00000004,
    /// Enable the `lsfMPTRequireAuth` capability flag (one-way; cannot be unset).
    TfMPTSetRequireAuth = 0x00000008,
    /// Enable the `lsfMPTCanEscrow` capability flag (one-way; cannot be unset).
    TfMPTSetCanEscrow = 0x00000010,
    /// Enable the `lsfMPTCanTrade` capability flag (one-way; cannot be unset).
    TfMPTSetCanTrade = 0x00000020,
    /// Enable the `lsfMPTCanTransfer` capability flag (one-way; cannot be unset).
    TfMPTSetCanTransfer = 0x00000040,
    /// Enable the `lsfMPTCanClawback` capability flag (one-way; cannot be unset).
    TfMPTSetCanClawback = 0x00000080,
    /// Enable the `lsfMPTCanHoldConfidentialBalance` capability flag (one-way; XLS-96).
    TfMPTSetCanHoldConfidentialBalance = 0x00000100,
}

impl TryFrom<u32> for MPTokenIssuanceSetFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000001 => Ok(MPTokenIssuanceSetFlag::TfMPTLock),
            0x00000002 => Ok(MPTokenIssuanceSetFlag::TfMPTUnlock),
            0x00000004 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanLock),
            0x00000008 => Ok(MPTokenIssuanceSetFlag::TfMPTSetRequireAuth),
            0x00000010 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanEscrow),
            0x00000020 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanTrade),
            0x00000040 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanTransfer),
            0x00000080 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanClawback),
            0x00000100 => Ok(MPTokenIssuanceSetFlag::TfMPTSetCanHoldConfidentialBalance),
            _ => Err(()),
        }
    }
}

impl MPTokenIssuanceSetFlag {
    /// Returns true if this flag is a capability-enabling flag (`tfMPTSet*`).
    pub fn is_capability_flag(&self) -> bool {
        matches!(
            self,
            MPTokenIssuanceSetFlag::TfMPTSetCanLock
                | MPTokenIssuanceSetFlag::TfMPTSetRequireAuth
                | MPTokenIssuanceSetFlag::TfMPTSetCanEscrow
                | MPTokenIssuanceSetFlag::TfMPTSetCanTrade
                | MPTokenIssuanceSetFlag::TfMPTSetCanTransfer
                | MPTokenIssuanceSetFlag::TfMPTSetCanClawback
                | MPTokenIssuanceSetFlag::TfMPTSetCanHoldConfidentialBalance
        )
    }

    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & 0x00000001 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTLock);
        }
        if bits & 0x00000002 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTUnlock);
        }
        if bits & 0x00000004 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanLock);
        }
        if bits & 0x00000008 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetRequireAuth);
        }
        if bits & 0x00000010 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanEscrow);
        }
        if bits & 0x00000020 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanTrade);
        }
        if bits & 0x00000040 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanTransfer);
        }
        if bits & 0x00000080 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanClawback);
        }
        if bits & 0x00000100 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTSetCanHoldConfidentialBalance);
        }
        flags
    }
}

/// Modifies properties of an existing MPToken issuance, such as locking
/// or unlocking tokens at the issuance or individual holder level.
///
/// See MPTokenIssuanceSet:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset>`
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuanceSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, MPTokenIssuanceSetFlag>,
    /// The MPToken issuance ID to modify, encoded as a hex string.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,
    /// The holder whose tokens to lock/unlock. If omitted, the lock/unlock
    /// applies to the entire issuance.
    pub holder: Option<Cow<'a, str>>,
    /// Domain (Hash256) associated with this issuance, encoded as a 64-char hex string.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
    /// Arbitrary hex-encoded metadata for the issuance (mutable post-creation).
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
    /// Transfer fee to update, in hundredths of a basis point (0–50000).
    pub transfer_fee: Option<u16>,
    /// Bitmask of fields and capability flags to permanently lock. Additive only —
    /// ORed into the ledger object's `ImmutableFlags` field.
    #[serde(
        default,
        rename = "ImmutableFlags",
        with = "opt_lgr_obj_flags",
        skip_serializing_if = "Option::is_none"
    )]
    pub immutable_flags: Option<FlagCollection<MPTokenIssuanceImmutableFlag>>,
}

impl<'a> Model for MPTokenIssuanceSet<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_flag_error()?;
        self._get_mptoken_issuance_id_error()?;
        self._get_holder_error()?;
        self._get_domain_id_error()?;
        self._get_domain_id_and_holder_conflict()?;
        self._get_holder_equals_account_error()?;
        self._get_metadata_error()?;
        self._get_transfer_fee_error()?;
        self._get_immutable_flags_error()?;
        self._get_mutation_with_holder_error()?;
        self._get_mutation_with_lock_flags_error()?;
        self._get_no_op_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, MPTokenIssuanceSetFlag> for MPTokenIssuanceSet<'a> {
    fn has_flag(&self, flag: &MPTokenIssuanceSetFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, MPTokenIssuanceSetFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenIssuanceSetFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, MPTokenIssuanceSetFlag> for MPTokenIssuanceSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenIssuanceSetFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> MPTokenIssuanceSet<'a> {
    pub fn with_mptoken_issuance_id(mut self, id: Cow<'a, str>) -> Self {
        self.mptoken_issuance_id = id;
        self
    }

    pub fn with_holder(mut self, holder: Cow<'a, str>) -> Self {
        self.holder = Some(holder);
        self
    }

    pub fn with_domain_id(mut self, domain_id: Cow<'a, str>) -> Self {
        self.domain_id = Some(domain_id);
        self
    }

    pub fn with_mptoken_metadata(mut self, mptoken_metadata: Cow<'a, str>) -> Self {
        self.mptoken_metadata = Some(mptoken_metadata);
        self
    }

    pub fn with_transfer_fee(mut self, transfer_fee: u16) -> Self {
        self.transfer_fee = Some(transfer_fee);
        self
    }

    pub fn with_immutable_flags(mut self, flags: Vec<MPTokenIssuanceImmutableFlag>) -> Self {
        self.immutable_flags = Some(flags.into());
        self
    }

    pub fn with_flag(mut self, flag: MPTokenIssuanceSetFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    pub fn with_flags(mut self, flags: Vec<MPTokenIssuanceSetFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }

    fn _get_flag_error(&self) -> XRPLModelResult<()> {
        let has_lock = self.has_flag(&MPTokenIssuanceSetFlag::TfMPTLock);
        let has_unlock = self.has_flag(&MPTokenIssuanceSetFlag::TfMPTUnlock);
        // rippled preflight rejects only when both flags are set simultaneously.
        // No-flag submissions are valid (e.g. DomainID-only changes).
        if has_lock && has_unlock {
            return Err(XRPLModelException::InvalidFlagCombination {
                flag1: "TfMPTLock".into(),
                flag2: "TfMPTUnlock".into(),
            });
        }
        Ok(())
    }

    fn _get_mptoken_issuance_id_error(&self) -> XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())
    }

    fn _get_holder_error(&self) -> XRPLModelResult<()> {
        if let Some(holder) = self.holder.as_deref() {
            validate_holder_address(holder)?;
        }
        Ok(())
    }

    fn _get_domain_id_error(&self) -> XRPLModelResult<()> {
        if let Some(id) = &self.domain_id {
            validate_domain_id(id.as_ref())?;
        }
        Ok(())
    }

    fn _get_metadata_error(&self) -> XRPLModelResult<()> {
        if let Some(metadata) = &self.mptoken_metadata {
            // An empty string is valid for MPTokenIssuanceSet: it clears the field on-ledger.
            // Only non-empty values must be valid hex and within the size limit.
            if !metadata.is_empty() {
                validate_mpt_metadata(metadata.as_ref())?;
            }
        }
        Ok(())
    }

    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(fee) = self.transfer_fee {
            validate_transfer_fee(fee)?;
        }
        Ok(())
    }

    /// Capability-setting flags (`tfMPTSet*`), `MPTokenMetadata`, `TransferFee`, and
    /// `ImmutableFlags` cannot be combined with `Holder`. Those operations target the
    /// issuance as a whole, not an individual holder.
    fn _get_mutation_with_holder_error(&self) -> XRPLModelResult<()> {
        if self.holder.is_none() {
            return Ok(());
        }
        let has_capability_flag = self
            .common_fields
            .flags
            .0
            .iter()
            .any(|f| f.is_capability_flag());
        if has_capability_flag
            || self.mptoken_metadata.is_some()
            || self.transfer_fee.is_some()
            || self.immutable_flags.is_some()
        {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "holder",
                other_fields: &[
                    "tfMPTSet* flags, mptoken_metadata, transfer_fee, or immutable_flags \
                     (mutation ops cannot be combined with holder)",
                ],
            });
        }
        Ok(())
    }

    /// Lock/unlock flags (`tfMPTLock`/`tfMPTUnlock`) cannot be combined with capability-setting
    /// flags (`tfMPTSet*`), `MPTokenMetadata`, `TransferFee`, or `ImmutableFlags`.
    fn _get_mutation_with_lock_flags_error(&self) -> XRPLModelResult<()> {
        let has_lock = self.has_flag(&MPTokenIssuanceSetFlag::TfMPTLock);
        let has_unlock = self.has_flag(&MPTokenIssuanceSetFlag::TfMPTUnlock);
        if !has_lock && !has_unlock {
            return Ok(());
        }
        let has_capability_flag = self
            .common_fields
            .flags
            .0
            .iter()
            .any(|f| f.is_capability_flag());
        if has_capability_flag
            || self.mptoken_metadata.is_some()
            || self.transfer_fee.is_some()
            || self.immutable_flags.is_some()
        {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "tfMPTLock or tfMPTUnlock",
                other_fields: &[
                    "tfMPTSet* flags, mptoken_metadata, transfer_fee, or immutable_flags \
                     (lock/unlock cannot be combined with mutation ops)",
                ],
            });
        }
        Ok(())
    }

    /// `DomainID` and `Holder` are mutually exclusive.
    fn _get_domain_id_and_holder_conflict(&self) -> XRPLModelResult<()> {
        if self.domain_id.is_some() && self.holder.is_some() {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "domain_id",
                other_fields: &["holder (DomainID and Holder cannot both be set)"],
            });
        }
        Ok(())
    }

    /// `Holder` must not be the same address as `Account`.
    fn _get_holder_equals_account_error(&self) -> XRPLModelResult<()> {
        if let Some(holder) = &self.holder {
            if holder.as_ref() == self.common_fields.account.as_ref() {
                return Err(XRPLModelException::InvalidFieldCombination {
                    field: "holder",
                    other_fields: &["account (Holder cannot be the same as Account)"],
                });
            }
        }
        Ok(())
    }

    /// `ImmutableFlags`, when present, must be non-zero and use only known `tif*` bits.
    fn _get_immutable_flags_error(&self) -> XRPLModelResult<()> {
        if let Some(flags) = &self.immutable_flags {
            let bits: u32 = flags.0.iter().map(|f| f.clone() as u32).fold(0, |acc, v| acc | v);
            if bits == 0 || (bits & TIF_MPTOKENISSUANCE_IMMUTABLE_MASK) != 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "immutable_flags".into(),
                    expected: alloc::format!(
                        "non-zero value using only known tif* bits (mask 0x{:08X})",
                        TIF_MPTOKENISSUANCE_VALID_MASK
                    ),
                    found: alloc::format!("0x{bits:08X}"),
                });
            }
        }
        Ok(())
    }

    /// Reject no-op transactions — something must change on the ledger object.
    /// The effective flags value is the numeric OR of all flags; an all-false
    /// object-form is equivalent to numeric 0 (no flags set).
    fn _get_no_op_error(&self) -> XRPLModelResult<()> {
        // Compute the effective numeric flags value.
        let flags_bits: u32 = self
            .common_fields
            .flags
            .0
            .iter()
            .map(|f| *f as u32)
            .fold(0, |acc, v| acc | v);

        let has_any_change = flags_bits != 0
            || self.domain_id.is_some()
            || self.mptoken_metadata.is_some()
            || self.transfer_fee.is_some()
            || self.immutable_flags.is_some();

        // A Holder field alone (without a lock/unlock flag or mutation) does nothing.
        if !has_any_change {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "MPTokenIssuanceSet",
                other_fields: &[
                    "transaction does not change the state of the MPTokenIssuance ledger object \
                     (no flags, no field mutations)",
                ],
            });
        }
        Ok(())
    }
}

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

/// Maximum transfer fee value (50000 = 50.000%).
const MAX_MPT_TRANSFER_FEE_SET: u16 = 50000;

/// Validates that a transfer fee is within the allowed range (0–50000).
pub(crate) fn validate_transfer_fee(fee: u16) -> XRPLModelResult<()> {
    if fee > MAX_MPT_TRANSFER_FEE_SET {
        return Err(XRPLModelException::ValueTooHigh {
            field: "transfer_fee".into(),
            max: MAX_MPT_TRANSFER_FEE_SET as u32,
            found: fee as u32,
        });
    }
    Ok(())
}

/// Maximum MPT metadata byte length per XLS-89.
const MAX_MPT_METADATA_BYTES_SET: usize = 1024;

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
    if byte_len > MAX_MPT_METADATA_BYTES_SET {
        return Err(XRPLModelException::ValueTooLong {
            field: "mptoken_metadata".into(),
            max: MAX_MPT_METADATA_BYTES_SET,
            found: byte_len,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::models::Model;

    use super::*;
    use crate::utils::testing::test_constants::*;

    #[test]
    fn test_serde() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                fee: Some("10".into()),
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_GENESIS.into()),
            ..Default::default()
        };

        let json_str = serde_json::to_string(&txn).unwrap();
        let deserialized: MPTokenIssuanceSet = serde_json::from_str(&json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_lock_unlock_conflict() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![
                    MPTokenIssuanceSetFlag::TfMPTLock,
                    MPTokenIssuanceSetFlag::TfMPTUnlock,
                ]
                .into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_builder_pattern() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_mptoken_issuance_id("00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into())
        .with_holder(ACCOUNT_GENESIS.into())
        .with_flag(MPTokenIssuanceSetFlag::TfMPTLock)
        .with_fee("12".into());

        assert_eq!(
            txn.mptoken_issuance_id.as_ref(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58"
        );
        assert_eq!(txn.holder.as_deref(), Some(ACCOUNT_GENESIS));
        assert!(txn.has_flag(&MPTokenIssuanceSetFlag::TfMPTLock));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_no_flag_is_no_op_rejected() {
        // A transaction with no flags and no field changes is a no-op — rejected.
        // (A Holder field alone with no lock/unlock flag is also a no-op.)
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_domain_id_only_change_is_valid() {
        // A DomainID-only change with no flags is not a no-op — it mutates the ledger object.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            domain_id: Some(
                "AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233".into(),
            ),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_unlock_only_is_ok() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTUnlock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_lock_only_is_ok() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_invalid_mptoken_issuance_id_length() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            // 32 hex chars, invalid (must be 48).
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_invalid_mptoken_issuance_id_non_hex() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            // Correct length, but contains a non-hex char ('Z').
            mptoken_issuance_id: "Z0000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_invalid_holder_address() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some("not_a_classic_address".into()),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_flag_try_from_u32() {
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000001),
            Ok(MPTokenIssuanceSetFlag::TfMPTLock)
        );
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000002),
            Ok(MPTokenIssuanceSetFlag::TfMPTUnlock)
        );
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000004),
            Ok(MPTokenIssuanceSetFlag::TfMPTSetCanLock)
        );
        assert!(MPTokenIssuanceSetFlag::try_from(0x00000200).is_err());
    }

    #[test]
    fn test_flag_from_bits() {
        let flags = MPTokenIssuanceSetFlag::from_bits(0x00000003);
        assert_eq!(flags.len(), 2);
        assert!(flags.contains(&MPTokenIssuanceSetFlag::TfMPTLock));
        assert!(flags.contains(&MPTokenIssuanceSetFlag::TfMPTUnlock));

        let empty = MPTokenIssuanceSetFlag::from_bits(0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_transaction_trait_methods() {
        use crate::models::transactions::Transaction;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert_eq!(
            *txn.get_transaction_type(),
            TransactionType::MPTokenIssuanceSet
        );
        assert_eq!(txn.get_common_fields().account.as_ref(), ACCOUNT_ISSUER);
    }

    #[test]
    fn test_with_flags_builder() {
        use crate::models::transactions::Transaction;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        }
        .with_flags(vec![MPTokenIssuanceSetFlag::TfMPTLock]);

        assert!(txn.has_flag(&MPTokenIssuanceSetFlag::TfMPTLock));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_domain_id_wrong_length_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            domain_id: Some("AABBCCDD".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_set_transfer_fee_within_range_valid() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            transfer_fee: Some(1000),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_set_transfer_fee_too_high_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            transfer_fee: Some(50001),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_set_metadata_valid() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_set_metadata_non_hex_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mptoken_metadata: Some("GGGG".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_set_immutable_flags_serde_as_integer() {
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            immutable_flags: Some(
                vec![MPTokenIssuanceImmutableFlag::LsifMPTTransferFee].into(),
            ),
            ..Default::default()
        };
        let json = serde_json::to_string(&txn).unwrap();
        assert!(
            json.contains("\"ImmutableFlags\":131072"),
            "ImmutableFlags should serialize as integer 131072, got: {json}"
        );
        let roundtrip: MPTokenIssuanceSet = serde_json::from_str(&json).unwrap();
        assert_eq!(txn, roundtrip);
    }

    #[test]
    fn test_all_new_fields_builder() {
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_mptoken_issuance_id("00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into())
        .with_domain_id("AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233".into())
        .with_mptoken_metadata("CAFEBABE".into())
        .with_transfer_fee(500)
        .with_immutable_flags(vec![MPTokenIssuanceImmutableFlag::LsifMPTMetadata]);

        assert_eq!(
            txn.domain_id.as_deref(),
            Some("AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233")
        );
        assert_eq!(txn.mptoken_metadata.as_deref(), Some("CAFEBABE"));
        assert_eq!(txn.transfer_fee, Some(500));
        assert!(txn.immutable_flags.is_some());
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_capability_flag_is_valid() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTSetCanTransfer].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_capability_flag_with_holder_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTSetCanTransfer].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_GENESIS.into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_lock_flag_with_capability_flag_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![
                    MPTokenIssuanceSetFlag::TfMPTLock,
                    MPTokenIssuanceSetFlag::TfMPTSetCanTransfer,
                ]
                .into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_lock_flag_with_metadata_rejected() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_flag_try_from_capability_flags() {
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000004),
            Ok(MPTokenIssuanceSetFlag::TfMPTSetCanLock)
        );
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000040),
            Ok(MPTokenIssuanceSetFlag::TfMPTSetCanTransfer)
        );
        assert_eq!(
            MPTokenIssuanceSetFlag::try_from(0x00000100),
            Ok(MPTokenIssuanceSetFlag::TfMPTSetCanHoldConfidentialBalance)
        );
        assert!(MPTokenIssuanceSetFlag::try_from(0x00000200).is_err());
    }

    #[test]
    fn test_flag_from_bits_capability() {
        let flags = MPTokenIssuanceSetFlag::from_bits(0x000001FC);
        assert_eq!(flags.len(), 7); // all tfMPTSet* flags
        assert!(flags.contains(&MPTokenIssuanceSetFlag::TfMPTSetCanLock));
        assert!(flags.contains(&MPTokenIssuanceSetFlag::TfMPTSetCanHoldConfidentialBalance));
    }

    // ── XLS-94D / DynamicMPT tests (mirrors JS MPTokenIssuanceSet.test.ts) ──

    #[test]
    fn test_multiple_capability_flags_valid() {
        // Setting multiple capability flags at once is valid.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![
                    MPTokenIssuanceSetFlag::TfMPTSetCanLock,
                    MPTokenIssuanceSetFlag::TfMPTSetRequireAuth,
                    MPTokenIssuanceSetFlag::TfMPTSetCanEscrow,
                    MPTokenIssuanceSetFlag::TfMPTSetCanTrade,
                    MPTokenIssuanceSetFlag::TfMPTSetCanTransfer,
                    MPTokenIssuanceSetFlag::TfMPTSetCanClawback,
                ]
                .into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_transfer_fee_and_metadata_mutation_valid() {
        // Updating TransferFee and MPTokenMetadata together is valid.
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            transfer_fee: Some(100),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_enable_can_transfer_and_set_transfer_fee_atomically_valid() {
        // XLS-94D allows enabling lsfMPTCanTransfer and setting a non-zero
        // TransferFee in the same transaction.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTSetCanTransfer].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            transfer_fee: Some(200),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_immutable_flags_alone_valid() {
        // ImmutableFlags alone (no other flags, no fields) is valid — permanently
        // locks a capability flag already set on the ledger.
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            immutable_flags: Some(
                vec![MPTokenIssuanceImmutableFlag::LsifMPTMetadata].into(),
            ),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_empty_metadata_clears_field_valid() {
        // An empty string for MPTokenMetadata is valid in MPTokenIssuanceSet:
        // rippled treats it as clearing the field (makeFieldAbsent).
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mptoken_metadata: Some("".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_immutable_flags_zero_rejected() {
        // ImmutableFlags present but zero must be rejected.
        let json = r#"{
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "TransactionType": "MPTokenIssuanceSet",
            "MPTokenIssuanceID": "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58",
            "ImmutableFlags": 0
        }"#;
        let txn: MPTokenIssuanceSet = serde_json::from_str(json).unwrap();
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_immutable_flags_reserved_bit_rejected() {
        // Bit 0x00000001 is reserved and not a valid tif* bit.
        let json = r#"{
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "TransactionType": "MPTokenIssuanceSet",
            "MPTokenIssuanceID": "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58",
            "ImmutableFlags": 1
        }"#;
        let txn: MPTokenIssuanceSet = serde_json::from_str(json).unwrap();
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_domain_id_and_holder_conflict_rejected() {
        // DomainID and Holder cannot both be set.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_GENESIS.into()),
            domain_id: Some(
                "AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233".into(),
            ),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_holder_equals_account_rejected() {
        // Holder cannot be the same address as Account.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_ISSUER.into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_no_op_rejected() {
        // A transaction with no flags and no mutations is a no-op — rejected.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_holder_with_metadata_mutation_rejected() {
        // Holder + metadata mutation is rejected.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_GENESIS.into()),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_unlock_with_metadata_rejected() {
        // tfMPTUnlock combined with metadata mutation is rejected.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTUnlock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }
}
