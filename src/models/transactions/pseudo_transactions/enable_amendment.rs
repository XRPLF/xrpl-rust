use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::amount::XRPAmount;
use crate::models::transactions::{CommonFields, FlagCollection, Memo, Signer};
use crate::models::{
    transactions::{Transaction, TransactionType},
    Model,
};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum EnableAmendmentFlag {
    /// Support for this amendment increased to at least 80% of trusted
    /// validators starting with this ledger version.
    TfGotMajority = 0x00010000,
    /// Support for this amendment decreased to less than 80% of trusted
    /// validators starting with this ledger version.
    TfLostMajority = 0x00020000,
}

/// See EnableAmendment:
/// `<https://xrpl.org/enableamendment.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EnableAmendment<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, EnableAmendmentFlag>,
    /// The custom fields for the EnableAmendment model.
    ///
    /// See EnableAmendment fields:
    /// `<https://xrpl.org/enableamendment.html#enableamendment-fields>`
    pub amendment: Cow<'a, str>,
    pub ledger_sequence: u32,
}

impl<'a> Model for EnableAmendment<'a> {}

impl<'a> Transaction<'a, EnableAmendmentFlag> for EnableAmendment<'a> {
    fn has_flag(&self, flag: &EnableAmendmentFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, EnableAmendmentFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, EnableAmendmentFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> EnableAmendment<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<EnableAmendmentFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amendment: Cow<'a, str>,
        ledger_sequence: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::EnableAmendment,
                account_txn_id,
                fee,
                Some(flags.unwrap_or_default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            amendment,
            ledger_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_serde_round_trip() {
        let txn = EnableAmendment::new(
            "rrrrrrrrrrrrrrrrrrrrrhoLvTp".into(),
            None,
            Some("10".into()),
            Some(FlagCollection::new(vec![EnableAmendmentFlag::TfGotMajority])),
            None,
            None,
            Some(1),
            None,
            None,
            None,
            "C1B8D934087225F509BEB5A8EC24447854713EE447D277F69545ABFA0E0FD490".into(),
            56865245,
        );
        let serialized = serde_json::to_string(&txn).unwrap();
        let deserialized: EnableAmendment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(txn, deserialized);
        assert!(serialized.contains("\"TransactionType\":\"EnableAmendment\""));
        assert!(serialized.contains("\"Amendment\""));
        assert!(serialized.contains("\"LedgerSequence\":56865245"));
    }

    #[test]
    fn test_has_flag() {
        let txn = EnableAmendment::new(
            "rrrrrrrrrrrrrrrrrrrrrhoLvTp".into(),
            None,
            None,
            Some(FlagCollection::new(vec![EnableAmendmentFlag::TfLostMajority])),
            None,
            None,
            None,
            None,
            None,
            None,
            "ABCD".into(),
            1,
        );
        assert!(txn.has_flag(&EnableAmendmentFlag::TfLostMajority));
        assert!(!txn.has_flag(&EnableAmendmentFlag::TfGotMajority));
        assert_eq!(txn.get_transaction_type(), &TransactionType::EnableAmendment);
    }
}
