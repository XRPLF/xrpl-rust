use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// A PermissionedDomainDelete transaction removes an existing permissioned
/// domain from the XRP Ledger. Only the owner of the domain can delete it.
///
/// See XLS-80 PermissionedDomains:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0080-permissioned-domains>`
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct PermissionedDomainDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the permissioned domain to delete.
    #[serde(rename = "DomainID")]
    pub domain_id: Cow<'a, str>,
}

impl<'a> Model for PermissionedDomainDelete<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        // DomainID is the 32-byte hash of the PermissionedDomain ledger entry,
        // serialized as 64 uppercase hex chars. Reuse the shared validator so the
        // rule stays in lockstep with PermissionedDomainSet.
        crate::models::transactions::permissioned_domain_set::validate_domain_id(
            self.domain_id.as_ref(),
        )?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for PermissionedDomainDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for PermissionedDomainDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> PermissionedDomainDelete<'a> {
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
        domain_id: Cow<'a, str>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::PermissionedDomainDelete,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                ..Default::default()
            },
            domain_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exceptions::XRPLModelException;

    /// Shared test account (a valid classic address).
    const TEST_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

    #[test]
    fn test_serde() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                fee: Some("10".into()),
                sequence: Some(1),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            domain_id: "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
        };

        let serialized = serde_json::to_string(&txn).unwrap();

        // Verify key fields are present
        assert!(serialized.contains("PermissionedDomainDelete"));
        assert!(serialized.contains("DomainID"));

        let deserialized: PermissionedDomainDelete = serde_json::from_str(&serialized).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_serde_json_format() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                fee: Some("12".into()),
                sequence: Some(5),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            domain_id: "AABB00112233445566778899AABB00112233445566778899AABB001122334455".into(),
        };

        let default_json_str = r#"{"Account":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh","TransactionType":"PermissionedDomainDelete","Fee":"12","Flags":0,"Sequence":5,"SigningPubKey":"","DomainID":"AABB00112233445566778899AABB00112233445566778899AABB001122334455"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: PermissionedDomainDelete =
            serde_json::from_str(default_json_str).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "AABB00112233445566778899AABB00112233445566778899AABB00112233445A".into(),
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(596447)
        .with_source_tag(42)
        .with_memo(Memo {
            memo_data: Some("deleting domain".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(txn.common_fields.account, TEST_ACCOUNT);
        assert_eq!(txn.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(txn.common_fields.sequence, Some(100));
        assert_eq!(txn.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(txn.common_fields.source_tag, Some(42));
        assert_eq!(txn.common_fields.memos.as_ref().unwrap().len(), 1);
        assert_eq!(
            txn.domain_id,
            "AABB00112233445566778899AABB00112233445566778899AABB00112233445A"
        );
    }

    #[test]
    fn test_default() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "AABB00112233445566778899AABB00112233445566778899AABB00112233445A".into(),
        };

        assert_eq!(txn.common_fields.account, TEST_ACCOUNT);
        assert_eq!(
            txn.common_fields.transaction_type,
            TransactionType::PermissionedDomainDelete
        );
        assert_eq!(
            txn.domain_id,
            "AABB00112233445566778899AABB00112233445566778899AABB00112233445A"
        );
        assert!(txn.common_fields.fee.is_none());
        assert!(txn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_new_constructor() {
        let txn = PermissionedDomainDelete::new(
            TEST_ACCOUNT.into(),
            None,
            Some("12".into()),
            Some(596447),
            None,
            Some(1),
            None,
            None,
            None,
            "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
        );

        assert_eq!(txn.common_fields.account, TEST_ACCOUNT);
        assert_eq!(
            txn.common_fields.transaction_type,
            TransactionType::PermissionedDomainDelete
        );
        assert_eq!(txn.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(txn.common_fields.sequence, Some(1));
        assert_eq!(txn.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(
            txn.domain_id,
            "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2"
        );
    }

    #[test]
    fn test_ticket_sequence() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "AABB00112233445566778899AABB00112233445566778899AABB00112233445A".into(),
        }
        .with_ticket_sequence(42)
        .with_fee("10".into());

        assert_eq!(txn.common_fields.ticket_sequence, Some(42));
        assert!(txn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_account_txn_id() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "AABB00112233445566778899AABB00112233445566778899AABB00112233445A".into(),
        }
        .with_account_txn_id("F1E2D3C4B5A69788".into())
        .with_fee("10".into())
        .with_sequence(50);

        assert_eq!(
            txn.common_fields.account_txn_id,
            Some("F1E2D3C4B5A69788".into())
        );
    }

    #[test]
    fn test_invalid_domain_id_rejected() {
        // DomainID must be exactly 64 hex chars (32-byte hash).
        let too_short = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "AABB0011".into(),
        };
        assert!(matches!(
            too_short.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { .. }
        ));

        // Correct length but non-hex chars.
        let non_hex = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "Z".repeat(64).into(),
        };
        assert!(matches!(
            non_hex.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { .. }
        ));
    }

    #[test]
    fn test_delete_all_zero_domain_id_rejected() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "0".repeat(64).into(),
        };

        assert!(matches!(
            txn.get_errors(),
            Err(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_valid_domain_id_accepted() {
        let txn = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: "A".repeat(64).into(),
        };
        assert!(txn.get_errors().is_ok());
    }
}
