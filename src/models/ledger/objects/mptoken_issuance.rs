use alloc::borrow::Cow;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::validate_mpt_amount_value, ledger::objects::LedgerEntryType, Model, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, LedgerObject};

const LSF_MPT_LOCKED_FLAG: u32 = 0x00000001;
const LSF_MPT_CAN_LOCK_FLAG: u32 = 0x00000002;
const LSF_MPT_REQUIRE_AUTH_FLAG: u32 = 0x00000004;
const LSF_MPT_CAN_ESCROW_FLAG: u32 = 0x00000008;
const LSF_MPT_CAN_TRADE_FLAG: u32 = 0x00000010;
const LSF_MPT_CAN_TRANSFER_FLAG: u32 = 0x00000020;
const LSF_MPT_CAN_CLAWBACK_FLAG: u32 = 0x00000040;

/// Flags that describe the persistent state of an `MPTokenIssuance` ledger object.
///
/// See MPTokenIssuance flags:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenIssuanceFlag {
    /// The issuance is currently locked.
    LsfMPTLocked = LSF_MPT_LOCKED_FLAG,
    /// The issuer can lock individual holders or the entire issuance.
    LsfMPTCanLock = LSF_MPT_CAN_LOCK_FLAG,
    /// Individual holders must be authorized before they can hold this token.
    LsfMPTRequireAuth = LSF_MPT_REQUIRE_AUTH_FLAG,
    /// This MPT can be held in escrow.
    LsfMPTCanEscrow = LSF_MPT_CAN_ESCROW_FLAG,
    /// This MPT can be traded on the DEX.
    LsfMPTCanTrade = LSF_MPT_CAN_TRADE_FLAG,
    /// This MPT can be transferred between accounts (other than issuer ↔ holder).
    LsfMPTCanTransfer = LSF_MPT_CAN_TRANSFER_FLAG,
    /// The issuer can claw back tokens from holders.
    LsfMPTCanClawback = LSF_MPT_CAN_CLAWBACK_FLAG,
}

impl TryFrom<u32> for MPTokenIssuanceFlag {
    type Error = XRPLModelException;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            value if value == MPTokenIssuanceFlag::LsfMPTLocked as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTLocked)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTCanLock as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTCanLock)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTRequireAuth as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTRequireAuth)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTCanEscrow as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTCanEscrow)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTCanTrade as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTCanTrade)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTCanTransfer as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTCanTransfer)
            }
            value if value == MPTokenIssuanceFlag::LsfMPTCanClawback as u32 => {
                Ok(MPTokenIssuanceFlag::LsfMPTCanClawback)
            }
            _ => Err(XRPLModelException::InvalidValue {
                field: "flags".into(),
                expected: "a known MPTokenIssuance ledger flag bit".into(),
                found: alloc::format!("0x{value:08X}"),
            }),
        }
    }
}

/// The `MPTokenIssuance` ledger object defines the properties and metadata of
/// a Multi-Purpose Token issuance on the XRP Ledger.
///
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MPTokenIssuance<'a> {
    /// The base fields for all ledger object models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, MPTokenIssuanceFlag>,
    /// The account that issued this MPT.
    pub issuer: Cow<'a, str>,
    /// An asset scale is the difference, in terms of orders of magnitude,
    /// between a standard unit and a corresponding fractional unit. The
    /// asset scale is a non-negative integer (0, 1, 2, ...) and defaults
    /// to 0.
    pub asset_scale: Option<u8>,
    /// The maximum number of MPTs that can exist at one time. If omitted,
    /// the maximum is currently limited to 2^63-1.
    pub maximum_amount: Option<Cow<'a, str>>,
    /// The total amount of MPTs of this issuance currently in circulation.
    /// This value increases when the issuer sends MPTs to a non-issuer, and
    /// decreases whenever the issuer receives MPTs.
    pub outstanding_amount: Cow<'a, str>,
    /// This value specifies the fee, in tenths of a basis point, charged by
    /// the issuer for secondary sales of the token, from 0 to 50,000
    /// inclusive (where 50,000 = 50%).
    pub transfer_fee: Option<u16>,
    /// Arbitrary metadata about this issuance, in hex format. The limit is
    /// 1024 bytes.
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
    /// The Sequence (or Ticket) number of the transaction that created this
    /// issuance, helping uniquely identify it.
    pub sequence: u32,
    /// A hint indicating which page of the owner directory links to this
    /// entry. Required by xrpld 2.3.0's `MPTokenIssuance` ledger entry
    /// definition (`sfOwnerNode`, `soeREQUIRED`).
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified
    /// this entry.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for MPTokenIssuance<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        if self.common_fields.index.is_none() && self.common_fields.ledger_index.is_none() {
            return Err(XRPLModelException::MissingField(
                "index or ledger_index".into(),
            ));
        }
        validate_mpt_amount_value("outstanding_amount", self.outstanding_amount.as_ref())?;
        if let Some(maximum_amount) = &self.maximum_amount {
            validate_mpt_amount_value("maximum_amount", maximum_amount.as_ref())?;
        }
        Ok(())
    }
}

impl<'a> LedgerObject<MPTokenIssuanceFlag> for MPTokenIssuance<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    use crate::models::FlagCollection;

    use super::*;
    use crate::models::transactions::test_fixtures::{
        ISSUER_ACCOUNT, LEDGER_OBJECT_INDEX, MPT_ISSUANCE_LEDGER_INDEX, PREVIOUS_TXN_ID,
        ZERO_HASH_256,
    };

    #[test]
    fn test_serde() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![MPTokenIssuanceFlag::LsfMPTCanTransfer]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(LEDGER_OBJECT_INDEX)),
                ledger_index: Some(Cow::from("87654321")),
            },
            issuer: ISSUER_ACCOUNT.into(),
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            outstanding_amount: "500000".into(),
            transfer_fee: Some(314),
            mptoken_metadata: Some("ABCDCAFE".into()),
            sequence: 42,
            owner_node: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 654321,
        };

        let serialized = serde_json::to_string(&issuance).unwrap();
        let deserialized: MPTokenIssuance = serde_json::from_str(&serialized).unwrap();
        assert_eq!(issuance, deserialized);
    }

    #[test]
    fn test_serde_requires_owner_node() {
        let json = alloc::format!(
            r#"{{"LedgerEntryType":"MPTokenIssuance","Flags":0,"Issuer":"{}","Sequence":1,"OutstandingAmount":"0","PreviousTxnID":"{}","PreviousTxnLgrSeq":1,"index":"{}"}}"#,
            ISSUER_ACCOUNT,
            PREVIOUS_TXN_ID,
            MPT_ISSUANCE_LEDGER_INDEX
        );

        let error = serde_json::from_str::<MPTokenIssuance>(&json).unwrap_err();
        assert!(error.to_string().contains("OwnerNode"));
    }

    #[test]
    fn test_ledger_entry_type() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(MPT_ISSUANCE_LEDGER_INDEX)),
                ledger_index: Some(Cow::from("1000000")),
            },
            issuer: ISSUER_ACCOUNT.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 100,
        };

        assert_eq!(
            issuance.get_ledger_entry_type(),
            LedgerEntryType::MPTokenIssuance
        );
    }

    #[test]
    fn test_minimal_issuance() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: None,
                ledger_index: Some(Cow::from("1000000")),
            },
            issuer: ISSUER_ACCOUNT.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: "0".into(),
            previous_txn_id: ZERO_HASH_256.into(),
            previous_txn_lgr_seq: 0,
        };

        assert!(issuance.validate().is_ok());
    }

    #[test]
    fn test_missing_index_and_ledger_index_error() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: None,
                ledger_index: None,
            },
            issuer: ISSUER_ACCOUNT.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: "0".into(),
            previous_txn_id: ZERO_HASH_256.into(),
            previous_txn_lgr_seq: 0,
        };

        assert!(issuance.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_invalid_amount_fields() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(MPT_ISSUANCE_LEDGER_INDEX)),
                ledger_index: None,
            },
            issuer: ISSUER_ACCOUNT.into(),
            asset_scale: None,
            maximum_amount: Some("9223372036854775808".into()),
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 100,
        };
        assert!(issuance.validate().is_err());

        let issuance = MPTokenIssuance {
            maximum_amount: None,
            outstanding_amount: "-1".into(),
            ..issuance
        };
        assert!(issuance.validate().is_err());
    }

    #[test]
    fn test_issuance_flag_try_from() {
        assert!(MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTLocked as u32).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTCanLock as u32).is_ok());
        assert!(
            MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTRequireAuth as u32).is_ok()
        );
        assert!(MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTCanEscrow as u32).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTCanTrade as u32).is_ok());
        assert!(
            MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTCanTransfer as u32).is_ok()
        );
        assert!(
            MPTokenIssuanceFlag::try_from(MPTokenIssuanceFlag::LsfMPTCanClawback as u32).is_ok()
        );
        assert!(MPTokenIssuanceFlag::try_from(
            (MPTokenIssuanceFlag::LsfMPTCanClawback as u32) << 1
        )
        .is_err());
    }
}
