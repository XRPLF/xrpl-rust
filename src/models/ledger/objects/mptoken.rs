use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::MPTAmount, ledger::objects::LedgerEntryType, Model, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, LedgerObject};

const LSF_MPT_LOCKED_FLAG: u32 = 0x0001;
const LSF_MPT_AUTHORIZED_FLAG: u32 = 0x0002;

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
    LsfMPTLocked = LSF_MPT_LOCKED_FLAG,
    /// This holder is authorized to hold the MPT. Set when the issuer
    /// authorizes the holder via `MPTokenAuthorize`.
    LsfMPTAuthorized = LSF_MPT_AUTHORIZED_FLAG,
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
}

impl<'a> Model for MPToken<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        if self.common_fields.index.is_none() && self.common_fields.ledger_index.is_none() {
            return Err(XRPLModelException::MissingField(
                "index or ledger_index".into(),
            ));
        }
        MPTAmount::new(self.mpt_amount.clone(), self.mptoken_issuance_id.clone()).get_errors()?;
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
    use crate::models::transactions::test_fixtures::{
        GENESIS_ACCOUNT, LEDGER_OBJECT_INDEX, MPTOKEN_LEDGER_INDEX, MPT_ISSUANCE_ID,
        PREVIOUS_TXN_ID,
    };

    #[test]
    fn test_serde() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![MPTokenFlag::LsfMPTAuthorized]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(LEDGER_OBJECT_INDEX)),
                ledger_index: None,
            },
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: MPT_ISSUANCE_ID.into(),
            mpt_amount: "1000".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 123456,
            owner_node: Some("0".into()),
        };

        let serialized = serde_json::to_string(&mptoken).unwrap();
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
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: MPT_ISSUANCE_ID.into(),
            mpt_amount: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
        };

        assert!(mptoken.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_invalid_mpt_amount() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(LEDGER_OBJECT_INDEX)),
                ledger_index: None,
            },
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: MPT_ISSUANCE_ID.into(),
            mpt_amount: "9223372036854775808".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
        };
        assert!(mptoken.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_invalid_mptoken_issuance_id() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(LEDGER_OBJECT_INDEX)),
                ledger_index: None,
            },
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: "not-a-valid-id".into(),
            mpt_amount: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
        };
        assert!(mptoken.validate().is_err());
    }

    #[test]
    fn test_validate_ok() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(LEDGER_OBJECT_INDEX)),
                ledger_index: None,
            },
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: MPT_ISSUANCE_ID.into(),
            mpt_amount: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 0,
            owner_node: None,
        };
        assert!(mptoken.validate().is_ok());
    }

    #[test]
    fn test_ledger_entry_type() {
        let mptoken = MPToken {
            common_fields: CommonFields {
                flags: FlagCollection(vec![]),
                ledger_entry_type: LedgerEntryType::MPToken,
                index: Some(Cow::from(MPTOKEN_LEDGER_INDEX)),
                ledger_index: Some(Cow::from("5000000")),
            },
            account: GENESIS_ACCOUNT.into(),
            mptoken_issuance_id: MPT_ISSUANCE_ID.into(),
            mpt_amount: "0".into(),
            previous_txn_id: PREVIOUS_TXN_ID.into(),
            previous_txn_lgr_seq: 4999998,
            owner_node: None,
        };

        assert_eq!(mptoken.get_ledger_entry_type(), LedgerEntryType::MPToken);
    }
}
