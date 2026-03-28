use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{ledger::objects::LedgerEntryType, Model, NoFlags};

use super::{CommonFields, LedgerObject};

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
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account that issued this MPT.
    pub issuer: Cow<'a, str>,
    /// The number of decimal places for this token's amounts.
    pub asset_scale: Option<u8>,
    /// The maximum amount of this token that can ever exist.
    pub maximum_amount: Option<Cow<'a, str>>,
    /// The total amount of this token currently in circulation.
    pub outstanding_amount: Cow<'a, str>,
    /// Transfer fee for this token (in hundredths of a basis point, 0-50000).
    pub transfer_fee: Option<u16>,
    /// Arbitrary metadata associated with this issuance (hex-encoded).
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
    /// The sequence number of the transaction that created this issuance.
    pub sequence: u32,
    /// The page in the owner's directory where this entry is located.
    pub owner_node: Option<Cow<'a, str>>,
    /// Hash of the most recent transaction that modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// Ledger index of the most recent transaction that modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for MPTokenIssuance<'a> {}

impl<'a> LedgerObject<NoFlags> for MPTokenIssuance<'a> {
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
                flags: FlagCollection(vec![]),
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
