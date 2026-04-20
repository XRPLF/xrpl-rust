use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::exceptions::XRPLModelException;
use crate::models::{
    transactions::{Credential, Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags};

use super::{CommonFields, CommonTransactionBuilder};

/// A PermissionedDomainSet transaction creates or updates a permissioned
/// domain on the XRP Ledger. A permissioned domain defines a set of
/// accepted credentials that grant access to restricted functionality.
///
/// When `domain_id` is `None`, a new domain is created. When `domain_id`
/// is provided, the existing domain is updated with the new set of
/// accepted credentials.
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
pub struct PermissionedDomainSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of an existing permissioned domain to update. If omitted,
    /// a new permissioned domain is created.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
    /// The list of credentials accepted by this domain. Each credential
    /// specifies an issuer and credential type.
    pub accepted_credentials: Vec<Credential>,
}

impl<'a> Model for PermissionedDomainSet<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        // XLS-80 mandates AcceptedCredentials contain between 1 and 10 entries.
        if self.accepted_credentials.is_empty() {
            return Err(XRPLModelException::MissingField(
                "AcceptedCredentials".into(),
            ));
        }
        if self.accepted_credentials.len() > 10 {
            return Err(XRPLModelException::ValueTooLong {
                field: "AcceptedCredentials".into(),
                max: 10,
                found: self.accepted_credentials.len(),
            });
        }
        for credential in &self.accepted_credentials {
            validate_credential(credential)?;
        }
        self.validate_currencies()
    }
}

/// Validate a `Credential` entry per XLS-80 / rippled `LedgerFormats.cpp`:
/// `Issuer` must be non-empty and `CredentialType` is an `sfBlob` (hex),
/// so it must be non-empty, even-length, hex-only, and at most 64 hex
/// chars (32 bytes, rippled's `MaxCredentialTypeLength`).
pub(crate) fn validate_credential(credential: &Credential) -> crate::models::XRPLModelResult<()> {
    if credential.issuer.is_empty() {
        return Err(XRPLModelException::MissingField("Credential.Issuer".into()));
    }
    let ct = &credential.credential_type;
    if ct.is_empty() {
        return Err(XRPLModelException::MissingField(
            "Credential.CredentialType".into(),
        ));
    }
    if ct.len() > 64 {
        return Err(XRPLModelException::ValueTooLong {
            field: "Credential.CredentialType".into(),
            max: 64,
            found: ct.len(),
        });
    }
    if !ct.len().is_multiple_of(2) || !ct.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(XRPLModelException::InvalidValue {
            field: "Credential.CredentialType".into(),
            expected: "even-length hex string (<=64 chars)".into(),
            found: ct.clone(),
        });
    }
    Ok(())
}

impl<'a> Transaction<'a, NoFlags> for PermissionedDomainSet<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for PermissionedDomainSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> PermissionedDomainSet<'a> {
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
        domain_id: Option<Cow<'a, str>>,
        accepted_credentials: Vec<Credential>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::PermissionedDomainSet,
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
            domain_id,
            accepted_credentials,
        }
    }

    /// Set the domain ID (for updating an existing domain).
    pub fn with_domain_id(mut self, domain_id: Cow<'a, str>) -> Self {
        self.domain_id = Some(domain_id);
        self
    }

    /// Set the accepted credentials list.
    pub fn with_accepted_credentials(mut self, credentials: Vec<Credential>) -> Self {
        self.accepted_credentials = credentials;
        self
    }

    /// Add a single credential to the accepted credentials list.
    pub fn with_credential(mut self, credential: Credential) -> Self {
        self.accepted_credentials.push(credential);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(1),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "rIssuer111111111111111111111".to_string(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        };

        let serialized = serde_json::to_string(&txn).unwrap();
        let deserialized: PermissionedDomainSet = serde_json::from_str(&serialized).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_serde_with_domain_id() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(2),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            domain_id: Some(
                "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
            ),
            accepted_credentials: vec![Credential {
                issuer: "rIssuer222222222222222222222".to_string(),
                credential_type: "414D4C".to_string(), // hex("AML")
            }],
        };

        let serialized = serde_json::to_string(&txn).unwrap();

        // Verify DomainID is present in serialized output
        assert!(serialized.contains("DomainID"));
        assert!(
            serialized.contains("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2")
        );

        let deserialized: PermissionedDomainSet = serde_json::from_str(&serialized).unwrap();
        assert_eq!(txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(596447)
        .with_source_tag(42)
        .with_credential(Credential {
            issuer: "rIssuer333333333333333333333".to_string(),
            credential_type: "4B5943".to_string(), // hex("KYC")
        });

        assert_eq!(
            txn.common_fields.account,
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );
        assert_eq!(txn.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(txn.common_fields.sequence, Some(100));
        assert_eq!(txn.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(txn.common_fields.source_tag, Some(42));
        assert_eq!(txn.accepted_credentials.len(), 1);
        assert!(txn.domain_id.is_none());
    }

    #[test]
    fn test_default() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            txn.common_fields.account,
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );
        assert_eq!(
            txn.common_fields.transaction_type,
            TransactionType::PermissionedDomainSet
        );
        assert!(txn.domain_id.is_none());
        assert!(txn.accepted_credentials.is_empty());
        assert!(txn.common_fields.fee.is_none());
        assert!(txn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_with_credentials() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(5),
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![
                Credential {
                    issuer: "rIssuerA".to_string(),
                    credential_type: "4B5943".to_string(), // hex("KYC")
                },
                Credential {
                    issuer: "rIssuerB".to_string(),
                    credential_type: "414D4C".to_string(), // hex("AML")
                },
                Credential {
                    issuer: "rIssuerC".to_string(),
                    credential_type: "41434352454449544544".to_string(), // hex("ACCREDITED")
                },
            ],
        };

        assert_eq!(txn.accepted_credentials.len(), 3);
        assert_eq!(txn.accepted_credentials[0].issuer, "rIssuerA".to_string());
        assert_eq!(
            txn.accepted_credentials[1].credential_type,
            "414D4C".to_string()
        );
        assert_eq!(
            txn.accepted_credentials[2].credential_type,
            "41434352454449544544".to_string()
        );
    }

    #[test]
    fn test_update_domain() {
        let domain_id =
            "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".to_string();
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(10),
                ..Default::default()
            },
            domain_id: Some(domain_id.clone().into()),
            accepted_credentials: vec![Credential {
                issuer: "rNewIssuer".to_string(),
                credential_type: "5645524946494544".to_string(), // hex("VERIFIED")
            }],
        };

        assert_eq!(txn.domain_id, Some(domain_id.into()));
        assert_eq!(txn.accepted_credentials.len(), 1);
    }

    #[test]
    fn test_create_domain() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "rIssuer111111111111111111111".to_string(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        };

        // Creating a new domain means domain_id is None
        assert!(txn.domain_id.is_none());
        assert_eq!(txn.accepted_credentials.len(), 1);
    }

    #[test]
    fn test_new_constructor() {
        let txn = PermissionedDomainSet::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            None,
            Some("12".into()),
            Some(596447),
            None,
            Some(1),
            None,
            None,
            None,
            None,
            vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        );

        assert_eq!(
            txn.common_fields.account,
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
        );
        assert_eq!(
            txn.common_fields.transaction_type,
            TransactionType::PermissionedDomainSet
        );
        assert_eq!(txn.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(txn.common_fields.sequence, Some(1));
        assert_eq!(txn.common_fields.last_ledger_sequence, Some(596447));
        assert!(txn.domain_id.is_none());
        assert_eq!(txn.accepted_credentials.len(), 1);
    }

    #[test]
    fn test_with_domain_id_builder() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_domain_id("AABB0011".into())
        .with_accepted_credentials(vec![Credential {
            issuer: "rIssuer".to_string(),
            credential_type: "4B5943".to_string(), // hex("KYC")
        }]);

        assert_eq!(txn.domain_id, Some("AABB0011".into()));
        assert_eq!(txn.accepted_credentials.len(), 1);
    }

    #[test]
    fn test_with_memo() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_fee("10".into())
        .with_sequence(1)
        .with_memo(Memo {
            memo_data: Some("creating domain".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_credential(Credential {
            issuer: "rIssuer".to_string(),
            credential_type: "4B5943".to_string(), // hex("KYC")
        });

        assert_eq!(txn.common_fields.memos.as_ref().unwrap().len(), 1);
        assert_eq!(txn.accepted_credentials.len(), 1);
    }

    #[test]
    fn test_empty_credentials_rejected() {
        // XLS-80 mandates AcceptedCredentials has 1..=10 entries; empty must fail validation.
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                fee: Some("10".into()),
                sequence: Some(1),
                ..Default::default()
            },
            domain_id: Some("AABB0011".into()),
            accepted_credentials: vec![],
        };

        let result = txn.get_errors();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            XRPLModelException::MissingField("AcceptedCredentials".into())
        );
    }

    #[test]
    fn test_too_many_credentials_rejected() {
        // XLS-80 caps AcceptedCredentials at 10 entries.
        let credentials: Vec<Credential> = (0..11)
            .map(|_| Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "4B5943".to_string(),
            })
            .collect();
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: credentials,
        };

        let result = txn.get_errors();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            XRPLModelException::ValueTooLong {
                max: 10,
                found: 11,
                ..
            }
        ));
    }

    #[test]
    fn test_non_hex_credential_type_rejected() {
        // CredentialType is an sfBlob; non-hex values must fail validation.
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "KYC".to_string(), // not hex
            }],
        };
        let result = txn.get_errors();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            XRPLModelException::InvalidValue { .. }
        ));
    }

    #[test]
    fn test_credential_type_too_long_rejected() {
        // rippled MaxCredentialTypeLength = 32 bytes (64 hex chars).
        let too_long = "A".repeat(66); // 66 hex chars
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: too_long,
            }],
        };
        let result = txn.get_errors();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            XRPLModelException::ValueTooLong { max: 64, .. }
        ));
    }

    #[test]
    fn test_ticket_sequence() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_ticket_sequence(42)
        .with_fee("10".into())
        .with_credential(Credential {
            issuer: "rIssuer".to_string(),
            credential_type: "4B5943".to_string(), // hex("KYC")
        });

        assert_eq!(txn.common_fields.ticket_sequence, Some(42));
        assert!(txn.common_fields.sequence.is_none());
    }

    #[test]
    fn test_credential_empty_issuer_rejected() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "".to_string(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        };

        let result = txn.get_errors();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            XRPLModelException::MissingField("Credential.Issuer".into())
        );
    }

    #[test]
    fn test_credential_empty_credential_type_rejected() {
        let txn = PermissionedDomainSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: "rIssuer".to_string(),
                credential_type: "".to_string(),
            }],
        };

        let result = txn.get_errors();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            XRPLModelException::MissingField("Credential.CredentialType".into())
        );
    }
}
