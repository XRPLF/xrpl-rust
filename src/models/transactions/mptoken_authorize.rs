use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{Transaction, TransactionType},
    Model, XRPLModelResult,
};

use super::mptoken_issuance_set::{validate_holder_address, validate_mptoken_issuance_id};
use super::{CommonFields, CommonTransactionBuilder};

/// Transactions of the MPTokenAuthorize type support additional values
/// in the Flags field.
///
/// See MPTokenAuthorize flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize>`
#[derive(
    Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenAuthorizeFlag {
    /// If set, revokes authorization (deauthorize / opt out).
    TfMPTUnauthorize = 0x00000001,
}

impl TryFrom<u32> for MPTokenAuthorizeFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000001 => Ok(MPTokenAuthorizeFlag::TfMPTUnauthorize),
            _ => Err(()),
        }
    }
}

impl MPTokenAuthorizeFlag {
    pub fn from_bits(bits: u32) -> Vec<Self> {
        let mut flags = Vec::new();
        if bits & 0x00000001 != 0 {
            flags.push(MPTokenAuthorizeFlag::TfMPTUnauthorize);
        }
        flags
    }
}

/// Authorizes an account to hold tokens from an MPToken issuance, or
/// (when sent by the issuer) authorizes a holder to participate.
///
/// See MPTokenAuthorize:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenAuthorize<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, MPTokenAuthorizeFlag>,
    /// The MPToken issuance ID to authorize for, encoded as a hex string.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,
    /// The holder to authorize. Omitted when a holder opts in themselves;
    /// provided when the issuer authorizes a specific holder.
    pub holder: Option<Cow<'a, str>>,
}

impl<'a> Model for MPTokenAuthorize<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_mptoken_issuance_id(self.mptoken_issuance_id.as_ref())?;
        if let Some(holder) = self.holder.as_deref() {
            validate_holder_address(holder)?;
        }
        Ok(())
    }
}

impl<'a> Transaction<'a, MPTokenAuthorizeFlag> for MPTokenAuthorize<'a> {
    fn has_flag(&self, flag: &MPTokenAuthorizeFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, MPTokenAuthorizeFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenAuthorizeFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, MPTokenAuthorizeFlag> for MPTokenAuthorize<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, MPTokenAuthorizeFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> MPTokenAuthorize<'a> {
    pub fn with_mptoken_issuance_id(mut self, id: Cow<'a, str>) -> Self {
        self.mptoken_issuance_id = id;
        self
    }

    pub fn with_holder(mut self, holder: Cow<'a, str>) -> Self {
        self.holder = Some(holder);
        self
    }

    pub fn with_flag(mut self, flag: MPTokenAuthorizeFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    pub fn with_flags(mut self, flags: Vec<MPTokenAuthorizeFlag>) -> Self {
        self.common_fields.flags = flags.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::models::Model;

    use super::*;
    use crate::utils::testing::test_constants::*;

    #[test]
    fn test_serde() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                fee: Some("10".into()),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some(ACCOUNT_GENESIS.into()),
        };

        let json_str = serde_json::to_string(&txn).unwrap();
        let deserialized: MPTokenAuthorize = serde_json::from_str(&json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_holder_opt_in() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.holder.is_none());
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_mptoken_issuance_id("00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into())
        .with_holder(ACCOUNT_GENESIS.into())
        .with_flag(MPTokenAuthorizeFlag::TfMPTUnauthorize)
        .with_fee("12".into());

        assert_eq!(
            txn.mptoken_issuance_id.as_ref(),
            "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58"
        );
        assert_eq!(txn.holder.as_deref(), Some(ACCOUNT_GENESIS));
        assert!(txn.has_flag(&MPTokenAuthorizeFlag::TfMPTUnauthorize));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_deauthorize_flow() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_GENESIS.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                flags: vec![MPTokenAuthorizeFlag::TfMPTUnauthorize].into(),
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };

        assert!(txn.has_flag(&MPTokenAuthorizeFlag::TfMPTUnauthorize));
        assert!(txn.validate().is_ok());
    }

    #[test]
    fn test_invalid_mptoken_issuance_id() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            // 32 hex chars; must be 48.
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00".into(),
            ..Default::default()
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_invalid_holder_address() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            holder: Some("not_a_valid_address".into()),
        };

        assert!(txn.validate().is_err());
    }

    #[test]
    fn test_flag_try_from_u32() {
        assert_eq!(
            MPTokenAuthorizeFlag::try_from(0x00000001),
            Ok(MPTokenAuthorizeFlag::TfMPTUnauthorize)
        );
        assert!(MPTokenAuthorizeFlag::try_from(0x00000002).is_err());
        assert!(MPTokenAuthorizeFlag::try_from(0).is_err());
    }

    #[test]
    fn test_flag_from_bits() {
        let flags = MPTokenAuthorizeFlag::from_bits(0x00000001);
        assert_eq!(flags.len(), 1);
        assert!(flags.contains(&MPTokenAuthorizeFlag::TfMPTUnauthorize));

        let empty = MPTokenAuthorizeFlag::from_bits(0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_transaction_trait_methods() {
        use crate::models::transactions::Transaction;
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        };
        assert_eq!(
            *txn.get_transaction_type(),
            TransactionType::MPTokenAuthorize
        );
        assert_eq!(txn.get_common_fields().account.as_ref(), ACCOUNT_ISSUER);
    }

    #[test]
    fn test_with_flags_builder() {
        let txn = MPTokenAuthorize {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            ..Default::default()
        }
        .with_flags(vec![MPTokenAuthorizeFlag::TfMPTUnauthorize]);

        assert!(txn.has_flag(&MPTokenAuthorizeFlag::TfMPTUnauthorize));
    }
}
