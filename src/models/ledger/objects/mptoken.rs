use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{ledger::objects::LedgerEntryType, Model, XRPLModelException, XRPLModelResult};

use super::{CommonFields, LedgerObject};

/// Ledger-object flags for the `MPToken` object.
///
/// See `MPToken` flags:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptoken>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenFlag {
    /// This holder's MPToken balance is locked.
    LsfMPTLocked = 0x0001,
    /// This holder is authorized to hold the MPT. Set when the issuer
    /// authorizes the holder via `MPTokenAuthorize`.
    LsfMPTAuthorized = 0x0002,
    /// This MPToken is held by an AMM account. Set by the protocol;
    /// matches `lsfMPTAMM` in rippled `LedgerFormats.h`.
    LsfMPTAMM = 0x0004,
}

/// The `MPToken` ledger object represents a single account's holdings of a
/// specific Multi-Purpose Token issuance.
///
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptoken>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MPToken<'a> {
    /// The base fields for all ledger object models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, MPTokenFlag>,
    /// The owner (holder) of these MPTs.
    pub account: Cow<'a, str>,
    /// The `MPTokenIssuance` identifier.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,
    /// The amount of tokens currently held by the owner. The minimum is 0
    /// and the maximum is 2^63-1.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified
    /// this entry.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// A hint indicating which page of the owner directory links to this
    /// entry, in case the directory consists of multiple pages.
    pub owner_node: Option<Cow<'a, str>>,
    /// The amount of this MPT currently locked in escrow or by other
    /// mechanisms. Present only when the TokenEscrow amendment is active.
    pub locked_amount: Option<Cow<'a, str>>,
}

impl<'a> Model for MPToken<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        if self.common_fields.index.is_none() && self.common_fields.ledger_index.is_none() {
            return Err(XRPLModelException::MissingField(
                "index or ledger_index".into(),
            ));
        }
        Ok(())
    }
}

impl<'a> LedgerObject<MPTokenFlag> for MPToken<'a> {
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
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![MPTokenFlag::LsfMPTAuthorized]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: None,
            },
            account: ACCOUNT_GENESIS.into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mpt_amount: "1000".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 123456,
            owner_node: Some("0".into()),
            locked_amount: None,
        };

        let serialized = serde_json::to_string(&mptoken).unwrap();
        let deserialized: MPToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mptoken, deserialized);
    }

    #[test]
    fn test_serde_with_locked_amount() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: None,
            },
            account: ACCOUNT_GENESIS.into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mpt_amount: "500".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 123456,
            owner_node: None,
            locked_amount: Some("250".into()),
        };

        let serialized = serde_json::to_string(&mptoken).unwrap();
        assert!(
            serialized.contains("\"LockedAmount\":\"250\""),
            "LockedAmount must serialize as PascalCase: {serialized}"
        );
        let deserialized: MPToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mptoken, deserialized);
    }

    #[test]
    fn test_missing_index_and_ledger_index_error() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: None,
                ledger_index: None,
            },
            account: ACCOUNT_GENESIS.into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mpt_amount: "0".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
            locked_amount: None,
        };

        assert!(mptoken.validate().is_err());
    }

    #[test]
    fn test_validate_ok() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: None,
            },
            account: ACCOUNT_GENESIS.into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mpt_amount: "0".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
            locked_amount: None,
        };
        assert!(mptoken.validate().is_ok());
    }

    #[test]
    fn test_ledger_entry_type() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(
                    "CF9421C5E0A80C7BC5F52A3566CCBD2E8F14C3DA1E65F3F3AB1EC5B5A3BDFEA",
                )),
                ledger_index: Some(Cow::from("5000000")),
            },
            account: ACCOUNT_GENESIS.into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58".into(),
            mpt_amount: "0".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 4999998,
            owner_node: None,
            locked_amount: None,
        };

        assert_eq!(mptoken.get_ledger_entry_type(), LedgerEntryType::MPToken);
    }

    #[test]
    fn test_lsf_mpt_amm_round_trip() {
        // An on-ledger MPToken held by an AMM has Flags: 4 (lsfMPTAMM).
        // Deserializing must produce LsfMPTAMM; reserializing must restore Flags: 4.
        // Prior to adding LsfMPTAMM the variant was unknown and silently became 0.
        let json = r#"{
            "LedgerEntryType": "MPToken",
            "Flags": 4,
            "index": "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "MPTokenIssuanceID": "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58",
            "MPTAmount": "0",
            "PreviousTxnID": "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879",
            "PreviousTxnLgrSeq": 0
        }"#;

        let mptoken: MPToken = serde_json::from_str(json).unwrap();
        assert!(
            mptoken
                .common_fields
                .flags
                .0
                .contains(&MPTokenFlag::LsfMPTAMM),
            "expected LsfMPTAMM in flags, got {:?}",
            mptoken.common_fields.flags
        );

        // Round-trip: reserialize and confirm Flags is 4
        let reserialized = serde_json::to_string(&mptoken).unwrap();
        assert!(
            reserialized.contains("\"Flags\":4"),
            "expected Flags:4 after round-trip, got: {reserialized}"
        );
    }
}
