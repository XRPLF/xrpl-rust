use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{ledger::objects::LedgerEntryType, Model};

use super::{CommonFields, LedgerObject};

/// Ledger-object flags for the `MPTokenIssuance` object.
///
/// See `MPTokenIssuance` flags:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/mptokenissuance>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum MPTokenIssuanceFlag {
    /// The MPT issuance is locked; no holder can send or receive tokens.
    LsfMPTLocked = 0x0001,
    /// The issuer retains the ability to lock the issuance or individual
    /// holders.
    LsfMPTCanLock = 0x0002,
    /// Holders must be individually authorized before they can hold this MPT.
    LsfMPTRequireAuth = 0x0004,
    /// This MPT can be placed into escrows.
    LsfMPTCanEscrow = 0x0008,
    /// This MPT can be traded on the DEX.
    LsfMPTCanTrade = 0x0010,
    /// This MPT can be transferred between non-issuer holders.
    LsfMPTCanTransfer = 0x0020,
    /// The issuer can clawback MPTs from holders.
    LsfMPTCanClawback = 0x0040,
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
    /// The address of the account that controls both the issuance amounts
    /// and characteristics of a particular fungible token.
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
}

impl<'a> Model for MPTokenIssuance<'a> {}

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

    #[test]
    fn test_serde() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![MPTokenIssuanceFlag::LsfMPTCanTransfer]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: None,
            },
            issuer: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
        };

        let serialized = serde_json::to_string(&issuance).unwrap();
        let deserialized: MPTokenIssuance = serde_json::from_str(&serialized).unwrap();
        assert_eq!(issuance, deserialized);
    }

    #[test]
    fn test_ledger_entry_type() {
        let issuance = MPTokenIssuance {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPTokenIssuance,
                index: None,
                ledger_index: None,
            },
            issuer: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
                ledger_index: None,
            },
            issuer: "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
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
        };

        assert!(issuance.validate().is_ok());
    }
}
