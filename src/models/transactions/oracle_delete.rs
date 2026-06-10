use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::{Memo, Signer, Transaction, TransactionType};
use crate::models::{FlagCollection, Model, NoFlags, XRPLModelResult};

use super::{CommonFields, CommonTransactionBuilder};

/// An OracleDelete transaction removes an Oracle ledger entry.
///
/// See OracleDelete:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/oracledelete>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OracleDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// A unique identifier of the price oracle for the account.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
}

impl Model for OracleDelete<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for OracleDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for OracleDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> OracleDelete<'a> {
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
        oracle_document_id: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::OracleDelete,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                network_id: None,
                sequence,
                signers,
                signing_pub_key: None, // filled by the signing layer
                source_tag,
                ticket_sequence,
                txn_signature: None, // filled by the signing layer
            },
            oracle_document_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical test account used across all OracleDelete unit tests.
    const TEST_ACCOUNT: &str = "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW";
    const TEST_FEE: &str = "12";
    const TEST_SEQUENCE: u32 = 391;
    const TEST_LAST_LEDGER: u32 = 596447;
    const TEST_DOC_ID: u32 = 1;

    #[test]
    fn test_serde() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                fee: Some(TEST_FEE.into()),
                sequence: Some(TEST_SEQUENCE),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            oracle_document_id: TEST_DOC_ID,
        };

        let default_json_str = r#"{"Account":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","TransactionType":"OracleDelete","Fee":"12","Flags":0,"Sequence":391,"SigningPubKey":"","OracleDocumentID":1}"#;

        let serialized_string = serde_json::to_string(&oracle_delete)
            .expect("OracleDelete should serialize to JSON without error");
        let serialized_value = serde_json::to_value(&serialized_string)
            .expect("serialized OracleDelete string should be valid JSON");
        let default_json_value =
            serde_json::to_value(default_json_str).expect("expected JSON string is valid JSON");
        assert_eq!(serialized_value, default_json_value);

        let deserialized: OracleDelete = serde_json::from_str(default_json_str)
            .expect("OracleDelete should deserialize from expected JSON");
        assert_eq!(oracle_delete, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: TEST_DOC_ID,
        }
        .with_fee(TEST_FEE.into())
        .with_sequence(TEST_SEQUENCE)
        .with_last_ledger_sequence(TEST_LAST_LEDGER)
        .with_source_tag(42)
        .with_memo(Memo {
            memo_data: Some("deleting oracle".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(oracle_delete.oracle_document_id, TEST_DOC_ID);
        assert_eq!(oracle_delete.common_fields.fee.as_ref().unwrap().0, TEST_FEE);
        assert_eq!(oracle_delete.common_fields.sequence, Some(TEST_SEQUENCE));
        assert_eq!(
            oracle_delete.common_fields.last_ledger_sequence,
            Some(TEST_LAST_LEDGER)
        );
        assert_eq!(oracle_delete.common_fields.source_tag, Some(42));
        assert_eq!(oracle_delete.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: 5,
        };

        assert_eq!(oracle_delete.common_fields.account, TEST_ACCOUNT);
        assert_eq!(
            oracle_delete.common_fields.transaction_type,
            TransactionType::OracleDelete
        );
        assert_eq!(oracle_delete.oracle_document_id, 5);
        assert!(oracle_delete.common_fields.fee.is_none());
        assert!(oracle_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_new_constructor() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                fee: Some(TEST_FEE.into()),
                last_ledger_sequence: Some(TEST_LAST_LEDGER),
                sequence: Some(TEST_SEQUENCE),
                ..Default::default()
            },
            oracle_document_id: TEST_DOC_ID,
        };

        assert_eq!(
            oracle_delete.common_fields.transaction_type,
            TransactionType::OracleDelete
        );
        assert_eq!(oracle_delete.common_fields.fee, Some(TEST_FEE.into()));
        assert_eq!(oracle_delete.common_fields.sequence, Some(TEST_SEQUENCE));
        assert_eq!(
            oracle_delete.common_fields.last_ledger_sequence,
            Some(TEST_LAST_LEDGER)
        );
        assert_eq!(oracle_delete.oracle_document_id, TEST_DOC_ID);
    }

    #[test]
    fn test_transaction_type() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: 0,
        };

        assert_eq!(
            *oracle_delete.get_transaction_type(),
            TransactionType::OracleDelete
        );
    }

    #[test]
    fn test_ticket_sequence() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: 3,
        }
        .with_ticket_sequence(54321)
        .with_fee(TEST_FEE.into());

        assert_eq!(oracle_delete.common_fields.ticket_sequence, Some(54321));
        assert!(oracle_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_zero_document_id() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: 0,
        };

        assert_eq!(oracle_delete.oracle_document_id, 0);
    }

    #[test]
    fn test_max_document_id() {
        let oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: u32::MAX,
        };

        assert_eq!(oracle_delete.oracle_document_id, u32::MAX);
    }
}
