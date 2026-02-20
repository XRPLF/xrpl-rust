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
    /// If set, indicates that this MPT can be transferred to other accounts.
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
    /// The number of decimal places for the MPT value. Must be in the
    /// range 0-9. Defaults to 0 if not provided.
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
                Err(XRPLModelException::ValueTooHigh {
                    field: "transfer_fee".into(),
                    max: MAX_MPT_TRANSFER_FEE as u32,
                    found: transfer_fee as u32,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::models::Model;

    use super::*;

    #[test]
    fn test_serde() {
        let txn = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                account: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            transfer_fee: Some(MAX_MPT_TRANSFER_FEE),
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
}
