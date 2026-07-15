use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// Delete the DID (Decentralized Identifier) associated with the
/// sending account.
///
/// See DIDDelete:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/diddelete>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DIDDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
}

impl<'a> Model for DIDDelete<'a> {}

impl<'a> Transaction<'a, NoFlags> for DIDDelete<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for DIDDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> DIDDelete<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::DIDDelete,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        let tx = DIDDelete {
            common_fields: CommonFields {
                account: "rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex".into(),
                transaction_type: TransactionType::DIDDelete,
                ..Default::default()
            },
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_serialize() {
        let tx = DIDDelete {
            common_fields: CommonFields {
                account: "rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex".into(),
                transaction_type: TransactionType::DIDDelete,
                fee: Some("12".into()),
                sequence: Some(391),
                signing_pub_key: Some(
                    "0293A815C095DBA82FAC597A6BB9D338674DB93168156D84D18417AD509FFF5904".into(),
                ),
                ..Default::default()
            },
        };

        let expected_json = r#"{"Account":"rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex","TransactionType":"DIDDelete","Fee":"12","Flags":0,"Sequence":391,"SigningPubKey":"0293A815C095DBA82FAC597A6BB9D338674DB93168156D84D18417AD509FFF5904"}"#;

        let serialized = serde_json::to_string(&tx).unwrap();
        let expected_value = serde_json::to_value(expected_json).unwrap();
        let serialized_value = serde_json::to_value(&serialized).unwrap();
        assert_eq!(serialized_value, expected_value);

        let deserialized: DIDDelete = serde_json::from_str(expected_json).unwrap();
        assert_eq!(tx, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let tx = DIDDelete {
            common_fields: CommonFields {
                account: "rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex".into(),
                transaction_type: TransactionType::DIDDelete,
                ..Default::default()
            },
        }
        .with_fee("12".into())
        .with_sequence(391)
        .with_last_ledger_sequence(7108682);

        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(tx.common_fields.sequence, Some(391));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
    }

    #[test]
    fn test_default() {
        let tx = DIDDelete {
            common_fields: CommonFields {
                account: "rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex".into(),
                transaction_type: TransactionType::DIDDelete,
                ..Default::default()
            },
        };

        assert_eq!(
            tx.common_fields.account,
            "rp4pqYgrTAtdPHuZd1ZQWxrzx45jxYcZex"
        );
        assert_eq!(
            tx.common_fields.transaction_type,
            TransactionType::DIDDelete
        );
        assert!(tx.common_fields.fee.is_none());
        assert!(tx.common_fields.sequence.is_none());
    }
}
