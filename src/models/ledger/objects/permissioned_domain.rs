use crate::models::ledger::objects::LedgerEntryType;
use crate::models::transactions::Credential;
use crate::models::FlagCollection;
use crate::models::Model;
use crate::models::NoFlags;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `PermissionedDomain` ledger entry represents a permissioned domain
/// on the XRP Ledger. A permissioned domain defines a set of accepted
/// credentials that restrict access to certain functionality.
///
/// See XLS-80 PermissionedDomains:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0080-permissioned-domains>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PermissionedDomain<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account that owns this permissioned domain.
    pub owner: Cow<'a, str>,
    /// The list of credentials accepted by this domain.
    pub accepted_credentials: Vec<Credential>,
    /// The sequence number of the PermissionedDomainSet transaction that
    /// created this domain.
    pub sequence: u32,
    /// A hint indicating which page of the owner directory links to this object,
    /// in case the directory consists of multiple pages.
    pub owner_node: Option<Cow<'a, str>>,
    /// The identifying hash of the transaction that most recently modified
    /// this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for PermissionedDomain<'a> {}

impl<'a> LedgerObject<NoFlags> for PermissionedDomain<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> PermissionedDomain<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        accepted_credentials: Vec<Credential>,
        sequence: u32,
        owner_node: Option<Cow<'a, str>>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::PermissionedDomain,
                index,
                ledger_index,
            },
            owner,
            accepted_credentials,
            sequence,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let domain = PermissionedDomain::new(
            Some(Cow::from("ForTest")),
            None,
            Cow::from("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"),
            vec![
                Credential {
                    issuer: "rIssuerA".to_string(),
                    credential_type: "4B5943".to_string(), // hex("KYC")
                },
                Credential {
                    issuer: "rIssuerB".to_string(),
                    credential_type: "414D4C".to_string(), // hex("AML")
                },
            ],
            1,
            Some(Cow::from("0")),
            Cow::from("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2"),
            1000,
        );

        let serialized = serde_json::to_string(&domain).unwrap();

        // Assert PascalCase JSON keys so silent field renames are caught.
        assert!(serialized.contains("\"AcceptedCredentials\""));
        assert!(serialized.contains("\"PreviousTxnID\""));
        assert!(serialized.contains("\"PreviousTxnLgrSeq\""));
        assert!(serialized.contains("\"Owner\""));
        assert!(serialized.contains("\"OwnerNode\""));
        assert!(serialized.contains("\"Sequence\""));
        assert!(serialized.contains("\"LedgerEntryType\""));

        let deserialized: PermissionedDomain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(domain, deserialized);
    }

    #[test]
    fn test_serialize_without_owner_node() {
        let domain = PermissionedDomain::new(
            Some(Cow::from("IndexHash")),
            None,
            Cow::from("rOwnerAccount123"),
            vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
            5,
            None,
            Cow::from("DEADBEEF01234567DEADBEEF01234567DEADBEEF01234567DEADBEEF01234567"),
            500,
        );

        let serialized = serde_json::to_string(&domain).unwrap();

        // OwnerNode should not appear when None
        assert!(!serialized.contains("OwnerNode"));

        let deserialized: PermissionedDomain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(domain, deserialized);
    }

    #[test]
    fn test_ledger_entry_type() {
        let domain = PermissionedDomain::new(
            None,
            None,
            Cow::from("rOwner"),
            vec![],
            1,
            None,
            Cow::from("0000000000000000000000000000000000000000000000000000000000000000"),
            1,
        );

        assert_eq!(
            domain.get_ledger_entry_type(),
            LedgerEntryType::PermissionedDomain
        );
    }

    #[test]
    fn test_empty_credentials() {
        let domain = PermissionedDomain::new(
            None,
            None,
            Cow::from("rOwner"),
            vec![],
            10,
            None,
            Cow::from("AABB00112233445566778899AABB00112233445566778899AABB001122334455AA"),
            200,
        );

        assert!(domain.accepted_credentials.is_empty());

        let serialized = serde_json::to_string(&domain).unwrap();
        let deserialized: PermissionedDomain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(domain, deserialized);
    }

    #[test]
    fn test_fields() {
        let domain = PermissionedDomain::new(
            Some(Cow::from("TestIndex")),
            Some(Cow::from("TestLedgerIndex")),
            Cow::from("rOwnerXYZ"),
            vec![Credential {
                issuer: "rIssuerXYZ".to_string(),
                credential_type: "41434352454449544544".to_string(), // hex("ACCREDITED")
            }],
            42,
            Some(Cow::from("7")),
            Cow::from("1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF"),
            999,
        );

        assert_eq!(domain.owner, "rOwnerXYZ");
        assert_eq!(domain.sequence, 42);
        assert_eq!(domain.owner_node, Some(Cow::from("7")));
        assert_eq!(
            domain.previous_txn_id,
            "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF"
        );
        assert_eq!(domain.previous_txn_lgr_seq, 999);
        assert_eq!(domain.accepted_credentials.len(), 1);
        assert_eq!(domain.common_fields.index, Some(Cow::from("TestIndex")));
        assert_eq!(
            domain.common_fields.ledger_index,
            Some(Cow::from("TestLedgerIndex"))
        );
    }
}
