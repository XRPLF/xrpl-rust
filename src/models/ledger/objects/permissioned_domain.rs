use crate::models::ledger::objects::LedgerEntryType;
use crate::models::transactions::Credential;
use crate::models::FlagCollection;
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
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified
    /// this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most
    /// recently modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> LedgerObject<NoFlags> for PermissionedDomain<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> crate::models::Model for PermissionedDomain<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        use crate::core::addresscodec::is_valid_classic_address;
        use crate::models::exceptions::XRPLModelException;
        use crate::models::transactions::permissioned_domain_set::validate_accepted_credentials;

        if !is_valid_classic_address(&self.owner) {
            return Err(XRPLModelException::InvalidValue {
                field: "owner".into(),
                expected: "valid classic XRPL address".into(),
                found: self.owner.clone().into_owned(),
            });
        }
        // Same credential-list rules as PermissionedDomainSet (1..=10, valid, no dupes)
        // so the ledger object and its originating transaction validate identically.
        validate_accepted_credentials(&self.accepted_credentials)
    }
}

impl<'a> PermissionedDomain<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        accepted_credentials: Vec<Credential>,
        sequence: u32,
        owner_node: Cow<'a, str>,
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

    /// Shared test owner / credential issuer (a valid classic address).
    const TEST_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

    #[test]
    fn test_serialize() {
        let domain = PermissionedDomain {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::PermissionedDomain,
                index: Some(Cow::from("ForTest")),
                ledger_index: None,
            },
            owner: Cow::from(TEST_ACCOUNT),
            accepted_credentials: vec![
                Credential {
                    issuer: "rIssuerA".to_string(),
                    credential_type: "4B5943".to_string(), // hex("KYC")
                },
                Credential {
                    issuer: "rIssuerB".to_string(),
                    credential_type: "414D4C".to_string(), // hex("AML")
                },
            ],
            sequence: 1,
            owner_node: Cow::from("0"),
            previous_txn_id: Cow::from(
                "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2",
            ),
            previous_txn_lgr_seq: 1000,
        };

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
    fn test_ledger_entry_type() {
        let domain = PermissionedDomain {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::PermissionedDomain,
                index: None,
                ledger_index: None,
            },
            owner: Cow::from("rOwner"),
            accepted_credentials: vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "4B5943".to_string(),
            }],
            sequence: 1,
            owner_node: Cow::from("0"),
            previous_txn_id: Cow::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            previous_txn_lgr_seq: 1,
        };

        assert_eq!(
            domain.get_ledger_entry_type(),
            LedgerEntryType::PermissionedDomain
        );
    }

    #[test]
    fn test_fields() {
        let domain = PermissionedDomain {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::PermissionedDomain,
                index: Some(Cow::from("TestIndex")),
                ledger_index: Some(Cow::from("TestLedgerIndex")),
            },
            owner: Cow::from("rOwnerXYZ"),
            accepted_credentials: vec![Credential {
                issuer: "rIssuerXYZ".to_string(),
                credential_type: "41434352454449544544".to_string(), // hex("ACCREDITED")
            }],
            sequence: 42,
            owner_node: Cow::from("7"),
            previous_txn_id: Cow::from(
                "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF",
            ),
            previous_txn_lgr_seq: 999,
        };

        assert_eq!(domain.owner, "rOwnerXYZ");
        assert_eq!(domain.sequence, 42);
        assert_eq!(domain.owner_node, Cow::from("7"));
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

    use crate::models::exceptions::XRPLModelException;
    use crate::models::Model;

    /// A valid PermissionedDomain ledger object (real classic addresses) for validation tests.
    fn valid_domain(credentials: Vec<Credential>) -> PermissionedDomain<'static> {
        PermissionedDomain {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::PermissionedDomain,
                index: None,
                ledger_index: None,
            },
            owner: Cow::from(TEST_ACCOUNT),
            accepted_credentials: credentials,
            sequence: 1,
            owner_node: Cow::from("0"),
            previous_txn_id: Cow::from(
                "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2",
            ),
            previous_txn_lgr_seq: 1000,
        }
    }

    fn kyc() -> Credential {
        Credential {
            issuer: TEST_ACCOUNT.to_string(),
            credential_type: "4B5943".to_string(),
        }
    }

    #[test]
    fn test_get_errors_valid() {
        assert!(valid_domain(vec![kyc()]).get_errors().is_ok());
    }

    #[test]
    fn test_get_errors_invalid_owner_rejected() {
        let mut domain = valid_domain(vec![kyc()]);
        domain.owner = Cow::from("not-an-address");
        assert!(matches!(
            domain.get_errors(),
            Err(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_get_errors_empty_credentials_rejected() {
        assert!(matches!(
            valid_domain(vec![]).get_errors(),
            Err(XRPLModelException::MissingField(_))
        ));
    }

    #[test]
    fn test_get_errors_duplicate_credentials_rejected() {
        // Shares the transaction's dedup rule (case-insensitive CredentialType).
        let dup_lower = Credential {
            issuer: TEST_ACCOUNT.to_string(),
            credential_type: "4b5943".to_string(),
        };
        assert!(matches!(
            valid_domain(vec![kyc(), dup_lower]).get_errors(),
            Err(XRPLModelException::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_get_errors_too_many_credentials_rejected() {
        let creds: Vec<Credential> = (0..11)
            .map(|i| Credential {
                issuer: TEST_ACCOUNT.to_string(),
                credential_type: alloc::format!("{:06X}", i),
            })
            .collect();
        assert!(matches!(
            valid_domain(creds).get_errors(),
            Err(XRPLModelException::ValueTooLong { .. })
        ));
    }
}
