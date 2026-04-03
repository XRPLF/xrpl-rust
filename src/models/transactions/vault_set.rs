use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{FlagCollection, Model, NoFlags, XRPLModelResult};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Update the settings of an existing vault on the XRP Ledger (XLS-65).
///
/// Only the vault owner can submit this transaction. It allows updating
/// optional metadata fields such as data, assets maximum, and domain ID.
///
/// See VaultSet transaction:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VaultSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the vault to update (256-bit hex string).
    #[serde(rename = "VaultID")]
    pub vault_id: Cow<'a, str>,
    /// Arbitrary hex-encoded data associated with the vault.
    pub data: Option<Cow<'a, str>>,
    /// The maximum amount of assets the vault can hold, as a string-encoded integer.
    pub assets_maximum: Option<Cow<'a, str>>,
    /// The domain ID associated with the vault.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
}

impl Model for VaultSet<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for VaultSet<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for VaultSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> VaultSet<'a> {
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
        data: Option<Cow<'a, str>>,
        assets_maximum: Option<Cow<'a, str>>,
        domain_id: Option<Cow<'a, str>>,
    ) -> VaultSet<'a> {
        VaultSet {
            common_fields: CommonFields::new(
                account,
                TransactionType::VaultSet,
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
            data,
            assets_maximum,
            domain_id,
        }
    }

    /// Set the data field.
    pub fn with_data(mut self, data: Cow<'a, str>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the assets maximum field.
    pub fn with_assets_maximum(mut self, assets_maximum: Cow<'a, str>) -> Self {
        self.assets_maximum = Some(assets_maximum);
        self
    }

    /// Set the domain ID field.
    pub fn with_domain_id(mut self, domain_id: Cow<'a, str>) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

    #[test]
    fn test_serde() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rVaultOwner123".into(),
                transaction_type: TransactionType::VaultSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            data: Some("48656C6C6F".into()),
            assets_maximum: None,
            domain_id: None,
        };

        let json_str = r#"{"Account":"rVaultOwner123","TransactionType":"VaultSet","Flags":0,"SigningPubKey":"","VaultID":"A0000000000000000000000000000000000000000000000000000000DEADBEEF","Data":"48656C6C6F"}"#;

        // Serialize
        let serialized = serde_json::to_string(&vault_set).unwrap();
        assert_eq!(
            serde_json::to_value(&serialized).unwrap(),
            serde_json::to_value(json_str).unwrap()
        );

        // Deserialize
        let deserialized: VaultSet = serde_json::from_str(json_str).unwrap();
        assert_eq!(vault_set, deserialized);
    }

    #[test]
    fn test_serde_all_optional_fields() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rVaultOwnerAll456".into(),
                transaction_type: TransactionType::VaultSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            data: Some("48656C6C6F".into()),
            assets_maximum: Some("2000000000".into()),
            domain_id: Some(
                "D0000000000000000000000000000000000000000000000000000000DEADBEEF".into(),
            ),
        };

        let serialized = serde_json::to_string(&vault_set).unwrap();
        let deserialized: VaultSet = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault_set, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rVaultOwner123".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_data("48656C6C6F".into())
        .with_assets_maximum("2000000000".into())
        .with_domain_id("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into())
        .with_memo(Memo {
            memo_data: Some("updating vault settings".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(vault_set.vault_id, VAULT_ID);
        assert_eq!(vault_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_set.common_fields.sequence, Some(100));
        assert_eq!(vault_set.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(vault_set.common_fields.source_tag, Some(12345));
        assert_eq!(vault_set.data, Some("48656C6C6F".into()));
        assert_eq!(vault_set.assets_maximum, Some("2000000000".into()));
        assert_eq!(vault_set.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rVaultOwner789".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        };

        assert_eq!(vault_set.common_fields.account, "rVaultOwner789");
        assert_eq!(
            vault_set.common_fields.transaction_type,
            TransactionType::VaultSet
        );
        assert_eq!(vault_set.vault_id, VAULT_ID);
        assert!(vault_set.data.is_none());
        assert!(vault_set.assets_maximum.is_none());
        assert!(vault_set.domain_id.is_none());
        assert!(vault_set.common_fields.fee.is_none());
        assert!(vault_set.common_fields.sequence.is_none());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_set = VaultSet {
            common_fields: CommonFields {
                account: "rTicketVaultSet111".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        }
        .with_ticket_sequence(54321)
        .with_fee("12".into());

        assert_eq!(ticket_set.common_fields.ticket_sequence, Some(54321));
        assert!(ticket_set.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_set = VaultSet {
            common_fields: CommonFields {
                account: "rMultiMemoVaultSet222".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        }
        .with_memo(Memo {
            memo_data: Some("first update".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("second update".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_set.common_fields.memos.as_ref().unwrap().len(),
            2
        );
        assert_eq!(multi_memo_set.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_new_constructor() {
        let vault_set = VaultSet::new(
            "rNewVaultSetter333".into(),
            None,
            Some("12".into()),
            Some(7108682),
            None,
            Some(100),
            None,
            None,
            None,
            VAULT_ID.into(),
            Some("48656C6C6F".into()),
            Some("2000000000".into()),
            Some("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into()),
        );

        assert_eq!(vault_set.common_fields.account, "rNewVaultSetter333");
        assert_eq!(
            vault_set.common_fields.transaction_type,
            TransactionType::VaultSet
        );
        assert_eq!(vault_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_set.vault_id, VAULT_ID);
        assert_eq!(vault_set.data, Some("48656C6C6F".into()));
        assert_eq!(vault_set.assets_maximum, Some("2000000000".into()));
    }

    #[test]
    fn test_validate() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rValidateVaultSet444".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(300);

        assert!(vault_set.validate().is_ok());
    }

    #[test]
    fn test_update_data_only() {
        let vault_set = VaultSet {
            common_fields: CommonFields {
                account: "rDataOnlyUpdate555".into(),
                transaction_type: TransactionType::VaultSet,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            ..Default::default()
        }
        .with_data("4E6577446174614F6E6C79".into())
        .with_fee("12".into())
        .with_sequence(500);

        assert_eq!(vault_set.data, Some("4E6577446174614F6E6C79".into()));
        assert!(vault_set.assets_maximum.is_none());
        assert!(vault_set.domain_id.is_none());
        assert!(vault_set.validate().is_ok());
    }
}
