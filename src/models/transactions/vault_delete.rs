use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, Model, NoFlags, XRPLModelResult};

use super::vault_common::validate_vault_id;
use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Delete a vault from the XRP Ledger (XLS-65).
///
/// The vault must be empty (no remaining assets) before it can be deleted.
/// Only the vault owner can submit this transaction.
///
/// See VaultDelete transaction:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VaultDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the vault to delete (256-bit hex string).
    #[serde(rename = "VaultID")]
    pub vault_id: Cow<'a, str>,
}

impl Model for VaultDelete<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_vault_id(&self.vault_id)
    }
}

impl<'a> Transaction<'a, NoFlags> for VaultDelete<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for VaultDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> VaultDelete<'a> {
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
        vault_id: Cow<'a, str>,
    ) -> VaultDelete<'a> {
        VaultDelete {
            common_fields: CommonFields::new(
                account,
                TransactionType::VaultDelete,
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
            vault_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

    #[test]
    fn test_serde() {
        let vault_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rVaultOwner123".into(),
                transaction_type: TransactionType::VaultDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        };

        let json_str = r#"{"Account":"rVaultOwner123","TransactionType":"VaultDelete","Flags":0,"SigningPubKey":"","VaultID":"A0000000000000000000000000000000000000000000000000000000DEADBEEF"}"#;

        // Serialize
        let serialized = serde_json::to_string(&vault_delete).unwrap();
        assert_eq!(
            serde_json::to_value(&serialized).unwrap(),
            serde_json::to_value(json_str).unwrap()
        );

        // Deserialize
        let deserialized: VaultDelete = serde_json::from_str(json_str).unwrap();
        assert_eq!(vault_delete, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let vault_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rVaultOwner123".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("deleting vault".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(vault_delete.vault_id, VAULT_ID);
        assert_eq!(vault_delete.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_delete.common_fields.sequence, Some(100));
        assert_eq!(
            vault_delete.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(vault_delete.common_fields.source_tag, Some(12345));
        assert_eq!(vault_delete.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let vault_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rVaultOwner456".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        };

        assert_eq!(vault_delete.common_fields.account, "rVaultOwner456");
        assert_eq!(
            vault_delete.common_fields.transaction_type,
            TransactionType::VaultDelete
        );
        assert_eq!(vault_delete.vault_id, VAULT_ID);
        assert!(vault_delete.common_fields.fee.is_none());
        assert!(vault_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rTicketVaultDel789".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        }
        .with_ticket_sequence(54321)
        .with_fee("12".into());

        assert_eq!(ticket_delete.common_fields.ticket_sequence, Some(54321));
        assert!(ticket_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rMultiMemoVaultDel111".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        }
        .with_memo(Memo {
            memo_data: Some("first memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("second memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_delete
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_delete.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_new_constructor() {
        let vault_delete = VaultDelete::new(
            "rNewDeleter222".into(),
            None,
            Some("12".into()),
            Some(7108682),
            None,
            Some(100),
            None,
            None,
            None,
            VAULT_ID.into(),
        );

        assert_eq!(vault_delete.common_fields.account, "rNewDeleter222");
        assert_eq!(
            vault_delete.common_fields.transaction_type,
            TransactionType::VaultDelete
        );
        assert_eq!(vault_delete.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(
            vault_delete.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(vault_delete.common_fields.sequence, Some(100));
        assert_eq!(vault_delete.vault_id, VAULT_ID);
    }

    #[test]
    fn test_validate() {
        let vault_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rValidateVaultDel333".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        }
        .with_fee("12".into())
        .with_sequence(300);

        assert!(vault_delete.validate().is_ok());
    }

    #[test]
    fn test_account_txn_id() {
        let vault_delete = VaultDelete {
            common_fields: CommonFields {
                account: "rVaultDelTxnId444".into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
        }
        .with_account_txn_id("F1E2D3C4B5A69788".into())
        .with_fee("12".into())
        .with_sequence(500);

        assert_eq!(
            vault_delete.common_fields.account_txn_id,
            Some("F1E2D3C4B5A69788".into())
        );
    }
}
