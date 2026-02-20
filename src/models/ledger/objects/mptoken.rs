use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{ledger::objects::LedgerEntryType, Model, NoFlags};

use super::{CommonFields, LedgerObject};

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
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account that holds this MPT balance.
    pub account: Cow<'a, str>,
    /// The issuance ID identifying which MPT this balance belongs to.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mptoken_issuance_id: Cow<'a, str>,
    /// The amount of tokens held by this account.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: Cow<'a, str>,
    /// Hash of the most recent transaction that modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// Ledger index of the most recent transaction that modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The page in the owner's directory where this entry is located.
    pub owner_node: Option<Cow<'a, str>>,
}

impl<'a> Model for MPToken<'a> {}

impl<'a> LedgerObject<NoFlags> for MPToken<'a> {
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
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(
                    "BFA9BE27383FA315651E26FDE1FA30815C5A5D0544EE10EC33D3E92532993769",
                )),
                ledger_index: None,
            },
            account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00".into(),
            mpt_amount: "1000".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 123456,
            owner_node: Some("0".into()),
        };

        let serialized = serde_json::to_string(&mptoken).unwrap();
        let deserialized: MPToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mptoken, deserialized);
    }

    #[test]
    fn test_ledger_entry_type() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: None,
                ledger_index: None,
            },
            account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            mptoken_issuance_id: "00000001A407AF5856CEFBF81F3D4A00".into(),
            mpt_amount: "0".into(),
            previous_txn_id: "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"
                .into(),
            previous_txn_lgr_seq: 100,
            owner_node: None,
        };

        assert_eq!(mptoken.get_ledger_entry_type(), LedgerEntryType::MPToken);
    }
}
