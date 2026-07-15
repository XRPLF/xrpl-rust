use alloc::borrow::Cow;
use core::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::_serde::opt_lgr_obj_flags;
use crate::models::{
    ledger::objects::LedgerEntryType, FlagCollection, Model, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, LedgerObject};

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
    LsfMPTLocked = 0x00000001,
    /// The issuer can lock individual holders or the entire issuance.
    LsfMPTCanLock = 0x00000002,
    /// Individual holders must be authorized before they can hold this token.
    LsfMPTRequireAuth = 0x00000004,
    /// This MPT can be held in escrow.
    LsfMPTCanEscrow = 0x00000008,
    /// This MPT can be traded on the DEX.
    LsfMPTCanTrade = 0x00000010,
    /// This MPT can be transferred between accounts (other than issuer ↔ holder).
    LsfMPTCanTransfer = 0x00000020,
    /// The issuer can claw back tokens from holders.
    LsfMPTCanClawback = 0x00000040,
}

impl TryFrom<u32> for MPTokenIssuanceFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000001 => Ok(MPTokenIssuanceFlag::LsfMPTLocked),
            0x00000002 => Ok(MPTokenIssuanceFlag::LsfMPTCanLock),
            0x00000004 => Ok(MPTokenIssuanceFlag::LsfMPTRequireAuth),
            0x00000008 => Ok(MPTokenIssuanceFlag::LsfMPTCanEscrow),
            0x00000010 => Ok(MPTokenIssuanceFlag::LsfMPTCanTrade),
            0x00000020 => Ok(MPTokenIssuanceFlag::LsfMPTCanTransfer),
            0x00000040 => Ok(MPTokenIssuanceFlag::LsfMPTCanClawback),
            _ => Err(()),
        }
    }
}

/// Mutable-flags bitmask stored in `sfMutableFlags` on an `MPTokenIssuance` ledger object.
/// These bits indicate which properties the issuer may update after creation.
///
/// See MPTokenIssuanceMutable flags (rippled `LedgerFormats.h`).
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenIssuanceMutableFlag {
    /// The issuer may toggle the `lsfMPTCanLock` flag after creation.
    LsmfMPTCanMutateCanLock = 0x00000002,
    /// The issuer may toggle the `lsfMPTRequireAuth` flag after creation.
    LsmfMPTCanMutateRequireAuth = 0x00000004,
    /// The issuer may toggle the `lsfMPTCanEscrow` flag after creation.
    LsmfMPTCanMutateCanEscrow = 0x00000008,
    /// The issuer may toggle the `lsfMPTCanTrade` flag after creation.
    LsmfMPTCanMutateCanTrade = 0x00000010,
    /// The issuer may toggle the `lsfMPTCanTransfer` flag after creation.
    LsmfMPTCanMutateCanTransfer = 0x00000020,
    /// The issuer may toggle the `lsfMPTCanClawback` flag after creation.
    LsmfMPTCanMutateCanClawback = 0x00000040,
    /// The issuer may update the `MPTokenMetadata` field after creation.
    LsmfMPTCanMutateMetadata = 0x00010000,
    /// The issuer may update the `TransferFee` field after creation.
    LsmfMPTCanMutateTransferFee = 0x00020000,
}

impl TryFrom<u32> for MPTokenIssuanceMutableFlag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x00000002 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateCanLock),
            0x00000004 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateRequireAuth),
            0x00000008 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateCanEscrow),
            0x00000010 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateCanTrade),
            0x00000020 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateCanTransfer),
            0x00000040 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateCanClawback),
            0x00010000 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateMetadata),
            0x00020000 => Ok(MPTokenIssuanceMutableFlag::LsmfMPTCanMutateTransferFee),
            _ => Err(()),
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
    /// entry.
    pub owner_node: Option<Cow<'a, str>>,
    /// The identifying hash of the transaction that most recently modified
    /// this entry.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// Bitmask of which fields the issuer may mutate after creation.
    /// Stored as `sfMutableFlags` on-ledger.
    #[serde(
        default,
        with = "opt_lgr_obj_flags",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutable_flags: Option<FlagCollection<MPTokenIssuanceMutableFlag>>,
    /// The total amount of this MPT currently locked in escrow or by other
    /// mechanisms across all holders. Present only when the TokenEscrow
    /// amendment is active.
    pub locked_amount: Option<Cow<'a, str>>,
}

impl<'a> Model for MPTokenIssuance<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        if self.common_fields.index.is_none() && self.common_fields.ledger_index.is_none() {
            return Err(XRPLModelException::MissingField(
                "index or ledger_index".into(),
            ));
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
    use alloc::vec;

    use crate::models::FlagCollection;

    use super::*;
    use crate::utils::testing::test_constants::*;

    #[test]
    fn test_serde() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![MPTokenIssuanceFlag::LsfMPTCanTransfer]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: Some(Cow::from("87654321")),
            },
            issuer: ACCOUNT_ISSUER.into(),
            asset_scale: Some(2),
            maximum_amount: Some("1000000".into()),
            outstanding_amount: "500000".into(),
            transfer_fee: Some(314),
            mptoken_metadata: Some("CAFEBABE".into()),
            sequence: 42,
            owner_node: Some("0".into()),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 654321,
            mutable_flags: Some(FlagCollection(vec![
                MPTokenIssuanceMutableFlag::LsmfMPTCanMutateTransferFee,
            ])),
            locked_amount: None,
        };

        let serialized = serde_json::to_string(&issuance).unwrap();
        // MutableFlags must serialize as an integer (rippled format), not an array.
        assert!(
            serialized.contains("\"MutableFlags\":131072"),
            "MutableFlags should serialize as integer 131072, got: {serialized}"
        );
        let deserialized: MPTokenIssuance = serde_json::from_str(&serialized).unwrap();
        assert_eq!(issuance, deserialized);
    }

    #[test]
    fn test_ledger_entry_type() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(
                    "A44128B79CAB60A1C97A72F5A4B0F43F04ABBE65B8B1C6AC24CF27E6DEA3B2A",
                )),
                ledger_index: Some(Cow::from("1000000")),
            },
            issuer: ACCOUNT_ISSUER.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: None,
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 100,
            mutable_flags: None,
            locked_amount: None,
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
            issuer: ACCOUNT_ISSUER.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: None,
            previous_txn_id: "0000000000000000000000000000000000000000000000000000000000000000"
                .into(),
            previous_txn_lgr_seq: 0,
            mutable_flags: None,
            locked_amount: None,
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
            issuer: ACCOUNT_ISSUER.into(),
            asset_scale: None,
            maximum_amount: None,
            outstanding_amount: "0".into(),
            transfer_fee: None,
            mptoken_metadata: None,
            sequence: 1,
            owner_node: None,
            previous_txn_id: "0000000000000000000000000000000000000000000000000000000000000000"
                .into(),
            previous_txn_lgr_seq: 0,
            mutable_flags: None,
            locked_amount: None,
        };

        assert!(issuance.validate().is_err());
    }

    #[test]
    fn test_mutable_flag_variants() {
        assert!(
            MPTokenIssuanceMutableFlag::try_from(0x00020000).is_ok(),
            "LsmfMPTCanMutateTransferFee should parse"
        );
        assert!(
            MPTokenIssuanceMutableFlag::try_from(0x00010000).is_ok(),
            "LsmfMPTCanMutateMetadata should parse"
        );
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000001).is_err());
        // cover all remaining match arms
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000002).is_ok());
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000004).is_ok());
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000008).is_ok());
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000010).is_ok());
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000020).is_ok());
        assert!(MPTokenIssuanceMutableFlag::try_from(0x00000040).is_ok());
    }

    #[test]
    fn test_issuance_flag_try_from() {
        assert!(MPTokenIssuanceFlag::try_from(0x00000001).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000002).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000004).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000008).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000010).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000020).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000040).is_ok());
        assert!(MPTokenIssuanceFlag::try_from(0x00000080).is_err());
    }

    /// Regression: sfMutableFlags is SoeDefault in xrpld — it may be absent from
    /// server JSON. Previously missing `#[serde(default)]` would cause deserialization
    /// to fail with "missing field `MutableFlags`" for any on-ledger MPTokenIssuance
    /// that did not set mutable flags.
    #[test]
    fn test_deserialize_without_mutable_flags() {
        let json = r#"{
            "Flags": 0,
            "Issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "MPTokenIssuanceID": "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58",
            "OutstandingAmount": "0",
            "OwnerNode": "0000000000000000",
            "PreviousTxnID": "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890",
            "PreviousTxnLgrSeq": 1,
            "Sequence": 1,
            "LedgerEntryType": "MPTokenIssuance"
        }"#;
        let obj: MPTokenIssuance =
            serde_json::from_str(json).expect("must deserialize without MutableFlags key");
        assert!(
            obj.mutable_flags.is_none(),
            "absent MutableFlags should deserialize as None"
        );
    }
}
