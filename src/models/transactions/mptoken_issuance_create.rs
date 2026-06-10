use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{Transaction, TransactionType},
    Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder};

/// Maximum transfer fee value (50000 = 50.000%).
const MAX_MPT_TRANSFER_FEE: u16 = 50000;
/// Maximum MPT metadata byte length per XLS-89.
const MAX_MPT_METADATA_BYTES: usize = 1024;
const TF_MPT_CAN_LOCK_FLAG: u32 = 0x00000002;
const TF_MPT_REQUIRE_AUTH_FLAG: u32 = 0x00000004;
const TF_MPT_CAN_ESCROW_FLAG: u32 = 0x00000008;
const TF_MPT_CAN_TRADE_FLAG: u32 = 0x00000010;
const TF_MPT_CAN_TRANSFER_FLAG: u32 = 0x00000020;
const TF_MPT_CAN_CLAWBACK_FLAG: u32 = 0x00000040;

/// Transactions of the MPTokenIssuanceCreate type support additional values
/// in the Flags field.
///
/// See MPTokenIssuanceCreate flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenIssuanceCreateFlag {
    /// If set, indicates that the MPT can be locked both at an issuance
    /// and individual level.
    TfMPTCanLock = TF_MPT_CAN_LOCK_FLAG,
    /// If set, indicates that individual holders must be authorized before
    /// they can hold the MPT.
    TfMPTRequireAuth = TF_MPT_REQUIRE_AUTH_FLAG,
    /// If set, indicates that this MPT can be held in escrow.
    TfMPTCanEscrow = TF_MPT_CAN_ESCROW_FLAG,
    /// If set, indicates that this MPT can be traded on the DEX.
    TfMPTCanTrade = TF_MPT_CAN_TRADE_FLAG,
    /// If set, indicates that the MPT can be transferred between accounts.
    TfMPTCanTransfer = TF_MPT_CAN_TRANSFER_FLAG,
    /// If set, indicates that the issuer can claw back the MPT.
    TfMPTCanClawback = TF_MPT_CAN_CLAWBACK_FLAG,
}

impl TryFrom<u32> for MPTokenIssuanceCreateFlag {
    type Error = XRPLModelException;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == MPTokenIssuanceCreateFlag::TfMPTCanLock as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTCanLock)
            }
            value if value == MPTokenIssuanceCreateFlag::TfMPTRequireAuth as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTRequireAuth)
            }
            value if value == MPTokenIssuanceCreateFlag::TfMPTCanEscrow as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTCanEscrow)
            }
            value if value == MPTokenIssuanceCreateFlag::TfMPTCanTrade as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTCanTrade)
            }
            value if value == MPTokenIssuanceCreateFlag::TfMPTCanTransfer as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTCanTransfer)
            }
            value if value == MPTokenIssuanceCreateFlag::TfMPTCanClawback as u32 => {
                Ok(MPTokenIssuanceCreateFlag::TfMPTCanClawback)
            }
            _ => Err(XRPLModelException::InvalidValue {
                field: "flags".into(),
                expected: "a known MPTokenIssuanceCreate flag bit".into(),
                found: alloc::format!("0x{value:08X}"),
            }),
        }
    }
}

impl MPTokenIssuanceCreateFlag {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & MPTokenIssuanceCreateFlag::TfMPTCanLock as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanLock);
        }
        if bits & MPTokenIssuanceCreateFlag::TfMPTRequireAuth as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTRequireAuth);
        }
        if bits & MPTokenIssuanceCreateFlag::TfMPTCanEscrow as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanEscrow);
        }
        if bits & MPTokenIssuanceCreateFlag::TfMPTCanTrade as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanTrade);
        }
        if bits & MPTokenIssuanceCreateFlag::TfMPTCanTransfer as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanTransfer);
        }
        if bits & MPTokenIssuanceCreateFlag::TfMPTCanClawback as u32 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanClawback);
        }
        flags
    }
}

/// Creates a new MPToken issuance on the XRPL.
///
/// See MPTokenIssuanceCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate>`
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
pub struct MPTokenIssuanceCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, MPTokenIssuanceCreateFlag>,
    /// The number of decimal places for the MPT value. This is a UINT8
    /// and defaults to 0 if not provided.
    pub asset_scale: Option<u8>,
    /// Maximum supply of the MPT as a string-encoded unsigned 64-bit integer.
    pub maximum_amount: Option<Cow<'a, str>>,
    /// Transfer fee charged by the issuer for secondary sales, in hundredths
    /// of a basis point (0-50000, representing 0.000%-50.000%).
    pub transfer_fee: Option<u16>,
    /// Arbitrary hex-encoded metadata for the issuance.
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
}

impl<'a> Model for MPTokenIssuanceCreate<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_transfer_fee_error()?;
        self._get_transfer_fee_requires_flag_error()?;
        self._get_metadata_error()?;
        self._get_maximum_amount_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, MPTokenIssuanceCreateFlag> for MPTokenIssuanceCreate<'a> {
    fn has_flag(&self, flag: &MPTokenIssuanceCreateFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, MPTokenIssuanceCreateFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenIssuanceCreateFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, MPTokenIssuanceCreateFlag> for MPTokenIssuanceCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenIssuanceCreateFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> MPTokenIssuanceCreate<'a> {
    pub fn with_asset_scale(mut self, asset_scale: u8) -> Self {
        self.asset_scale = Some(asset_scale);
        self
    }

    pub fn with_maximum_amount(mut self, maximum_amount: Cow<'a, str>) -> Self {
        self.maximum_amount = Some(maximum_amount);
        self
    }

    pub fn with_transfer_fee(mut self, transfer_fee: u16) -> Self {
        self.transfer_fee = Some(transfer_fee);
        self
    }

    pub fn with_mptoken_metadata(mut self, mptoken_metadata: Cow<'a, str>) -> Self {
        self.mptoken_metadata = Some(mptoken_metadata);
        self
    }

    pub fn with_flag(mut self, flag: MPTokenIssuanceCreateFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    pub fn with_flags(mut self, flags: Vec<MPTokenIssuanceCreateFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }

    fn _get_transfer_fee_error(&self) -> XRPLModelResult<()> {
        if let Some(transfer_fee) = self.transfer_fee {
            if transfer_fee > MAX_MPT_TRANSFER_FEE {
                return Err(XRPLModelException::ValueTooHigh {
                    field: "transfer_fee".into(),
                    max: u32::from(MAX_MPT_TRANSFER_FEE),
                    found: u32::from(transfer_fee),
                });
            }
        }
        Ok(())
    }

    fn _get_transfer_fee_requires_flag_error(&self) -> XRPLModelResult<()> {
        if self
            .transfer_fee
            .is_some_and(|transfer_fee| transfer_fee > 0)
            && !self.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer)
        {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "transfer_fee",
                other_fields: &["flags (TfMPTCanTransfer must be set for non-zero transfer_fee)"],
            });
        }
        Ok(())
    }

    fn _get_maximum_amount_error(&self) -> XRPLModelResult<()> {
        if let Some(max_amount) = &self.maximum_amount {
            if max_amount.is_empty() || !max_amount.bytes().all(|b| b.is_ascii_digit()) {
                return Err(XRPLModelException::InvalidValueFormat {
                    field: "maximum_amount".into(),
                    format: "unsigned 64-bit integer string".into(),
                    found: max_amount.as_ref().into(),
                });
            }
            let value: u64 =
                max_amount
                    .parse()
                    .map_err(|error| XRPLModelException::InvalidValueFormat {
                        field: "maximum_amount".into(),
                        format: alloc::format!("unsigned 64-bit integer string ({error})"),
                        found: max_amount.as_ref().into(),
                    })?;
            if value == 0 || value > i64::MAX as u64 {
                return Err(XRPLModelException::InvalidValue {
                    field: "maximum_amount".into(),
                    expected: alloc::format!("1..={}", i64::MAX),
                    found: max_amount.as_ref().into(),
                });
            }
        }
        Ok(())
    }

    fn _get_metadata_error(&self) -> XRPLModelResult<()> {
        if let Some(metadata) = &self.mptoken_metadata {
            if metadata.is_empty()
                || metadata.len() % 2 != 0
                || !metadata.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(XRPLModelException::InvalidValueFormat {
                    field: "mptoken_metadata".into(),
                    format: "non-empty even-length ASCII hex string".into(),
                    found: metadata.as_ref().into(),
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::models::Model;

    use super::*;
    use crate::models::transactions::test_fixtures::{DESTINATION_ACCOUNT, ISSUER_ACCOUNT};

    #[test]
    fn test_serde() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                fee: Some("10".into()),
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            transfer_fee: Some(314),
            mptoken_metadata: Some("ABCD".into()),
        };

        let json_str = serde_json::to_string(&txn).unwrap();
        let deserialized: MPTokenIssuanceCreate = serde_json::from_str(&json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_transfer_fee_error() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: DESTINATION_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            transfer_fee: Some(50001),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
        assert_eq!(
            txn.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"transfer_fee\"` is defined above its maximum (max 50000, found 50001)"
        );
    }

    #[test]
    fn test_builder_pattern() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_asset_scale(6)
        .with_maximum_amount("999999999".into())
        .with_transfer_fee(100)
        .with_mptoken_metadata("ABCDCAFE".into())
        .with_flags(vec![
            MPTokenIssuanceCreateFlag::TfMPTCanTransfer,
            MPTokenIssuanceCreateFlag::TfMPTCanLock,
        ])
        .with_fee("12".into())
        .with_sequence(42);

        assert_eq!(txn.asset_scale, Some(6));
        assert_eq!(txn.maximum_amount.as_deref(), Some("999999999"));
        assert_eq!(txn.transfer_fee, Some(100));
        assert_eq!(txn.mptoken_metadata.as_deref(), Some("ABCDCAFE"));
        assert!(txn.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer));
        assert!(txn.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanLock));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_default() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(txn.asset_scale.is_none());
        assert!(txn.maximum_amount.is_none());
        assert!(txn.transfer_fee.is_none());
        assert!(txn.mptoken_metadata.is_none());
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_transfer_fee_at_max() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            transfer_fee: Some(MAX_MPT_TRANSFER_FEE),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_asset_scale_accepts_full_uint8_range() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            asset_scale: Some(u8::MAX),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_transfer_fee_without_can_transfer_flag_error() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            transfer_fee: Some(100),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_transfer_fee_with_can_transfer_flag_ok() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            transfer_fee: Some(100),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_metadata_too_long_error() {
        // 1025 bytes = 2050 hex chars
        let metadata = "AB".repeat(1025);
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mptoken_metadata: Some(metadata.into()),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_metadata_at_max_length_ok() {
        // exactly 1024 bytes = 2048 hex chars
        let metadata = "AB".repeat(1024);
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mptoken_metadata: Some(metadata.into()),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_metadata_odd_length_error() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mptoken_metadata: Some("ABC".into()),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_metadata_non_hex_error() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mptoken_metadata: Some("GGGG".into()),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_metadata_empty_string_error() {
        // xrpl.js: !isHex("") → false → invalid. Empty string must be rejected.
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mptoken_metadata: Some("".into()),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_transfer_fee_zero_without_flag_ok() {
        // xrpld only requires TfMPTCanTransfer when TransferFee is non-zero.
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            transfer_fee: Some(0),
            ..Default::default()
        };

        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_flag_try_from_u32() {
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTCanLock as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanLock)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTRequireAuth as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTRequireAuth)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTCanEscrow as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanEscrow)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTCanTrade as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanTrade)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTCanTransfer as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanTransfer)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(MPTokenIssuanceCreateFlag::TfMPTCanClawback as u32),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanClawback)
        );
        assert!(MPTokenIssuanceCreateFlag::try_from(
            (MPTokenIssuanceCreateFlag::TfMPTCanLock as u32) >> 1
        )
        .is_err());
        assert!(MPTokenIssuanceCreateFlag::try_from(
            (MPTokenIssuanceCreateFlag::TfMPTCanClawback as u32) << 1
        )
        .is_err());
    }

    #[test]
    fn test_flag_from_bits() {
        let flags = MPTokenIssuanceCreateFlag::from_bits(
            MPTokenIssuanceCreateFlag::TfMPTCanLock as u32
                | MPTokenIssuanceCreateFlag::TfMPTRequireAuth as u32
                | MPTokenIssuanceCreateFlag::TfMPTCanTransfer as u32,
        );
        assert_eq!(flags.len(), 3);
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTCanLock));
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTRequireAuth));
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer));

        let empty = MPTokenIssuanceCreateFlag::from_bits(0);
        assert!(empty.is_empty());

        let all = MPTokenIssuanceCreateFlag::from_bits(
            MPTokenIssuanceCreateFlag::TfMPTCanLock as u32
                | MPTokenIssuanceCreateFlag::TfMPTRequireAuth as u32
                | MPTokenIssuanceCreateFlag::TfMPTCanEscrow as u32
                | MPTokenIssuanceCreateFlag::TfMPTCanTrade as u32
                | MPTokenIssuanceCreateFlag::TfMPTCanTransfer as u32
                | MPTokenIssuanceCreateFlag::TfMPTCanClawback as u32,
        );
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_transaction_trait_methods() {
        use crate::models::transactions::Transaction;
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            *txn.get_transaction_type(),
            TransactionType::MPTokenIssuanceCreate
        );
        assert_eq!(txn.get_common_fields().account.as_ref(), ISSUER_ACCOUNT);
    }

    #[test]
    fn test_maximum_amount_zero_rejected() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: Some("0".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_maximum_amount_too_large_rejected() {
        // i64::MAX = 9223372036854775807; anything above is rejected by xrpld
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: Some("9223372036854775808".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_maximum_amount_at_max_is_ok() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: Some("9223372036854775807".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_maximum_amount_plus_prefix_rejected() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: Some("+1".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_maximum_amount_none_is_ok() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ISSUER_ACCOUNT.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: None,
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }
}
