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
    ledger::objects::mptoken_issuance::MPTokenIssuanceMutableFlag,
    transactions::{Transaction, TransactionType},
    FlagCollection, Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
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
}

impl TryFrom<u32> for MPTokenIssuanceSetFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000001 => Ok(MPTokenIssuanceSetFlag::TfMPTLock),
            0x00000002 => Ok(MPTokenIssuanceSetFlag::TfMPTUnlock),
            _ => Err(()),
        }
    }
}

impl MPTokenIssuanceSetFlag {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & 0x00000001 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTLock);
        }
        if bits & 0x00000002 != 0 {
            flags.push(MPTokenIssuanceSetFlag::TfMPTUnlock);
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
    /// Bitmask of which issuance fields are mutable after creation.
    /// Stored as a UInt32 on the wire; reuses the ledger-object mutable-flag enum.
    #[serde(
        default,
        with = "opt_lgr_obj_flags",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutable_flags: Option<FlagCollection<MPTokenIssuanceMutableFlag>>,
}

impl<'a> Model for MPTokenIssuanceSet<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_flag_error()?;
        self._get_mptoken_issuance_id_error()?;
        self._get_holder_error()?;
        self._get_domain_id_error()?;
        self._get_metadata_error()?;
        self._get_transfer_fee_error()?;
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

    pub fn with_mutable_flags(mut self, flags: Vec<MPTokenIssuanceMutableFlag>) -> Self {
        self.mutable_flags = Some(flags.into());
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
            validate_mpt_metadata(metadata.as_ref())?;
        }
        Ok(())
    }

    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(fee) = self.transfer_fee {
            validate_transfer_fee(fee)?;
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
    fn test_no_flag_is_valid() {
        // No-flag submissions are valid per rippled (e.g. DomainID-only changes).
        // rippled only rejects when both TfMPTLock and TfMPTUnlock are set simultaneously.
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.holder.is_none());
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
        assert!(MPTokenIssuanceSetFlag::try_from(0x00000004).is_err());
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
    fn test_domain_id_only_valid() {
        // No-flag DomainID-only update — referenced in the existing comment at _get_flag_error.
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
    fn test_set_mutable_flags_serde_as_integer() {
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceMutableFlag;
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mutable_flags: Some(
                vec![MPTokenIssuanceMutableFlag::LsmfMPTCanMutateTransferFee].into(),
            ),
            ..Default::default()
        };
        let json = serde_json::to_string(&txn).unwrap();
        assert!(
            json.contains("\"MutableFlags\":131072"),
            "MutableFlags should serialize as integer 131072, got: {json}"
        );
        let roundtrip: MPTokenIssuanceSet = serde_json::from_str(&json).unwrap();
        assert_eq!(txn, roundtrip);
    }

    #[test]
    fn test_all_new_fields_builder() {
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceMutableFlag;
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
        .with_mutable_flags(vec![MPTokenIssuanceMutableFlag::LsmfMPTCanMutateMetadata]);

        assert_eq!(
            txn.domain_id.as_deref(),
            Some("AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233")
        );
        assert_eq!(txn.mptoken_metadata.as_deref(), Some("CAFEBABE"));
        assert_eq!(txn.transfer_fee, Some(500));
        assert!(txn.mutable_flags.is_some());
        assert!(txn.validate().is_ok());
    }
}
