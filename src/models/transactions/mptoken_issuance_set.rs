use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::core::addresscodec::decode_classic_address;
use crate::models::{
    transactions::{Transaction, TransactionType},
    Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
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
}

impl<'a> Model for MPTokenIssuanceSet<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_flag_error()?;
        self._get_mptoken_issuance_id_error()?;
        self._get_holder_error()?;
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
        if has_lock && has_unlock {
            return Err(XRPLModelException::InvalidFlagCombination {
                flag1: "TfMPTLock".into(),
                flag2: "TfMPTUnlock".into(),
            });
        }
        // Rippled preflight requires exactly one of TfMPTLock / TfMPTUnlock
        // (DomainID modification is another allowed form, not yet modelled
        // here); reject the no-flag submission until that lands.
        if !has_lock && !has_unlock {
            return Err(XRPLModelException::ExpectedOneOf(&[
                "TfMPTLock",
                "TfMPTUnlock",
            ]));
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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::models::Model;

    use super::*;

    #[test]
    fn test_serde() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                fee: Some("10".into()),
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into()),
        };

        let json_str = serde_json::to_string(&txn).unwrap();
        let deserialized: MPTokenIssuanceSet = serde_json::from_str(&json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_lock_unlock_conflict() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_mptoken_issuance_id("00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into())
        .with_holder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into())
        .with_flag(MPTokenIssuanceSetFlag::TfMPTLock)
        .with_fee("12".into());

        assert_eq!(
            txn.mptoken_issuance_id.as_ref(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58"
        );
        assert_eq!(
            txn.holder.as_deref(),
            Some("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
        );
        assert!(txn.has_flag(&MPTokenIssuanceSetFlag::TfMPTLock));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_default_requires_flag() {
        // With neither TfMPTLock nor TfMPTUnlock set, rippled rejects the tx
        // in preflight. The model mirrors that (DomainID-only changes are not
        // yet modelled).
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.holder.is_none());
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_lock_only_is_ok() {
        let txn = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some("not_a_classic_address".into()),
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
}
