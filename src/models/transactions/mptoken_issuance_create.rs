use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::_serde::opt_lgr_obj_flags;
use crate::models::{
    ledger::objects::mptoken_issuance::MPTokenIssuanceMutableFlag,
    transactions::{Transaction, TransactionType},
    FlagCollection, Model, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};

use super::{
    mpt_common::{validate_domain_id, validate_mpt_metadata, validate_transfer_fee},
    CommonFields, CommonTransactionBuilder,
};

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
    TfMPTCanLock = 0x00000002,
    /// If set, indicates that individual holders must be authorized before
    /// they can hold the MPT.
    TfMPTRequireAuth = 0x00000004,
    /// If set, indicates that this MPT can be held in escrow.
    TfMPTCanEscrow = 0x00000008,
    /// If set, indicates that this MPT can be traded on the DEX.
    TfMPTCanTrade = 0x00000010,
    /// If set, indicates that the MPT can be transferred between accounts.
    TfMPTCanTransfer = 0x00000020,
    /// If set, indicates that the issuer can claw back the MPT.
    TfMPTCanClawback = 0x00000040,
}

impl TryFrom<u32> for MPTokenIssuanceCreateFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000002 => Ok(MPTokenIssuanceCreateFlag::TfMPTCanLock),
            0x00000004 => Ok(MPTokenIssuanceCreateFlag::TfMPTRequireAuth),
            0x00000008 => Ok(MPTokenIssuanceCreateFlag::TfMPTCanEscrow),
            0x00000010 => Ok(MPTokenIssuanceCreateFlag::TfMPTCanTrade),
            0x00000020 => Ok(MPTokenIssuanceCreateFlag::TfMPTCanTransfer),
            0x00000040 => Ok(MPTokenIssuanceCreateFlag::TfMPTCanClawback),
            _ => Err(()),
        }
    }
}

impl MPTokenIssuanceCreateFlag {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & 0x00000002 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanLock);
        }
        if bits & 0x00000004 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTRequireAuth);
        }
        if bits & 0x00000008 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanEscrow);
        }
        if bits & 0x00000010 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanTrade);
        }
        if bits & 0x00000020 != 0 {
            flags.push(MPTokenIssuanceCreateFlag::TfMPTCanTransfer);
        }
        if bits & 0x00000040 != 0 {
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
    /// Domain (Hash256) associated with this issuance, encoded as a 64-char hex string.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
    /// Bitmask of which issuance fields are mutable after creation.
    /// Stored as a UInt32 on the wire; reuses the ledger-object mutable-flag enum.
    #[serde(
        default,
        with = "opt_lgr_obj_flags",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutable_flags: Option<FlagCollection<MPTokenIssuanceMutableFlag>>,
}

impl<'a> Model for MPTokenIssuanceCreate<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_transfer_fee_error()?;
        self._get_transfer_fee_requires_flag_error()?;
        self._get_metadata_error()?;
        self._get_maximum_amount_error()?;
        self._get_domain_id_error()?;
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

    pub fn with_domain_id(mut self, domain_id: Cow<'a, str>) -> Self {
        self.domain_id = Some(domain_id);
        self
    }

    pub fn with_mutable_flags(mut self, flags: Vec<MPTokenIssuanceMutableFlag>) -> Self {
        self.mutable_flags = Some(flags.into());
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
            validate_transfer_fee(transfer_fee)?;
        }
        Ok(())
    }

    fn _get_transfer_fee_requires_flag_error(&self) -> XRPLModelResult<()> {
        // rippled only rejects a non-zero TransferFee without tfMPTCanTransfer:
        //   if (fee > 0u && !ctx.tx.isFlag(tfMPTCanTransfer)) return temMALFORMED
        // TransferFee: 0 is explicitly allowed even without the flag.
        if matches!(self.transfer_fee, Some(fee) if fee > 0)
            && !self.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer)
        {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "transfer_fee",
                other_fields: &["flags (TfMPTCanTransfer must be set when transfer_fee > 0)"],
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
                    .map_err(|_| XRPLModelException::InvalidValueFormat {
                        field: "maximum_amount".into(),
                        format: "unsigned 64-bit integer string".into(),
                        found: max_amount.as_ref().into(),
                    })?;
            // rippled rejects zero: if (n == 0) return temMALFORMED
            if value == 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "maximum_amount".into(),
                    expected: "non-zero unsigned integer string".into(),
                    found: max_amount.as_ref().into(),
                });
            }
            if value > i64::MAX as u64 {
                return Err(XRPLModelException::InvalidValue {
                    field: "maximum_amount".into(),
                    expected: alloc::format!("<= {}", i64::MAX),
                    found: max_amount.as_ref().into(),
                });
            }
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
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::models::Model;

    use super::super::mpt_common::MAX_MPT_TRANSFER_FEE;
    use super::*;
    use crate::utils::testing::test_constants::*;

    #[test]
    fn test_serde() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                fee: Some("10".into()),
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            transfer_fee: Some(314),
            mptoken_metadata: Some("ABCD".into()),
            ..Default::default()
        };

        let json_str = serde_json::to_string(&txn).unwrap();
        let deserialized: MPTokenIssuanceCreate = serde_json::from_str(&json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_transfer_fee_error() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
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
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_asset_scale(6)
        .with_maximum_amount("999999999".into())
        .with_transfer_fee(100)
        .with_mptoken_metadata("CAFEBABE".into())
        .with_flags(vec![
            MPTokenIssuanceCreateFlag::TfMPTCanTransfer,
            MPTokenIssuanceCreateFlag::TfMPTCanLock,
        ])
        .with_fee("12".into())
        .with_sequence(42);

        assert_eq!(txn.asset_scale, Some(6));
        assert_eq!(txn.maximum_amount.as_deref(), Some("999999999"));
        assert_eq!(txn.transfer_fee, Some(100));
        assert_eq!(txn.mptoken_metadata.as_deref(), Some("CAFEBABE"));
        assert!(txn.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer));
        assert!(txn.has_flag(&MPTokenIssuanceCreateFlag::TfMPTCanLock));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_default() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
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
        // TransferFee: 0 without TfMPTCanTransfer must be accepted.
        // rippled only rejects non-zero: if (fee > 0u && !ctx.tx.isFlag(tfMPTCanTransfer))
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
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
            MPTokenIssuanceCreateFlag::try_from(0x00000002),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanLock)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(0x00000004),
            Ok(MPTokenIssuanceCreateFlag::TfMPTRequireAuth)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(0x00000008),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanEscrow)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(0x00000010),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanTrade)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(0x00000020),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanTransfer)
        );
        assert_eq!(
            MPTokenIssuanceCreateFlag::try_from(0x00000040),
            Ok(MPTokenIssuanceCreateFlag::TfMPTCanClawback)
        );
        assert!(MPTokenIssuanceCreateFlag::try_from(0x00000001).is_err());
        assert!(MPTokenIssuanceCreateFlag::try_from(0x00000080).is_err());
    }

    #[test]
    fn test_flag_from_bits() {
        let flags = MPTokenIssuanceCreateFlag::from_bits(0x00000026);
        assert_eq!(flags.len(), 3);
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTCanLock));
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTRequireAuth));
        assert!(flags.contains(&MPTokenIssuanceCreateFlag::TfMPTCanTransfer));

        let empty = MPTokenIssuanceCreateFlag::from_bits(0);
        assert!(empty.is_empty());

        let all = MPTokenIssuanceCreateFlag::from_bits(0x0000007E);
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_transaction_trait_methods() {
        use crate::models::transactions::Transaction;
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            *txn.get_transaction_type(),
            TransactionType::MPTokenIssuanceCreate
        );
        assert_eq!(txn.get_common_fields().account.as_ref(), ACCOUNT_ISSUER);
    }

    #[test]
    fn test_maximum_amount_zero_rejected() {
        // rippled rejects MaximumAmount == 0: if (n == 0) return temMALFORMED
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
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
        // i64::MAX = 9223372036854775807; anything above is rejected by rippled
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: Some("9223372036854775807".into()),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_domain_id_accepted() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            domain_id: Some(
                "AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233".into(),
            ),
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_domain_id_wrong_length_rejected() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            domain_id: Some("AABBCCDD".into()), // too short
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_domain_id_non_hex_rejected() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            domain_id: Some(
                "ZABBCCDD00112233AABBCCDD00112233AABBCCDD00112233AABBCCDD00112233".into(),
            ),
            ..Default::default()
        };
        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_mutable_flags_serde_round_trip() {
        use crate::models::ledger::objects::mptoken_issuance::MPTokenIssuanceMutableFlag;
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            mutable_flags: Some(vec![MPTokenIssuanceMutableFlag::LsmfMPTCanMutateMetadata].into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&txn).unwrap();
        // MutableFlags must serialize as integer, not array
        assert!(
            json.contains("\"MutableFlags\":65536"),
            "MutableFlags should serialize as integer 65536, got: {json}"
        );
        let roundtrip: MPTokenIssuanceCreate = serde_json::from_str(&json).unwrap();
        assert_eq!(txn, roundtrip);
    }

    #[test]
    fn test_maximum_amount_plus_prefix_rejected() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
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
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            maximum_amount: None,
            ..Default::default()
        };
        assert!(txn.validate().is_ok());
    }
}
