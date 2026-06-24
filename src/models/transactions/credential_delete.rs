use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, XRPLModelException, XRPLModelResult,
};
use crate::models::{FlagCollection, NoFlags};

use super::CommonTransactionBuilder;

/// A CredentialDelete transaction deletes a credential object.
///
/// See CredentialDelete:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0070-credentials>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialDelete<'a> {
    /// The base fields for all transaction models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The subject of the credential. At least one of `subject` or `issuer` must
    /// be provided. When omitted, Account implicitly fills the subject role.
    pub subject: Option<Cow<'a, str>>,
    /// The issuer of the credential. At least one of `subject` or `issuer` must
    /// be provided. When omitted, Account implicitly fills the issuer role.
    pub issuer: Option<Cow<'a, str>>,
    /// A hex-encoded value identifying the credential type from this issuer.
    pub credential_type: Cow<'a, str>,
}

impl<'a> Model for CredentialDelete<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_subject_or_issuer_error()?;
        self._get_credential_type_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for CredentialDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CredentialDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> CredentialDelete<'a> {
    #[allow(clippy::too_many_arguments)]
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
        subject: Option<Cow<'a, str>>,
        issuer: Option<Cow<'a, str>>,
        credential_type: Cow<'a, str>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CredentialDelete,
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
            subject,
            issuer,
            credential_type,
        }
    }
}

impl<'a> CredentialDeleteError for CredentialDelete<'a> {
    fn _get_subject_or_issuer_error(&self) -> XRPLModelResult<()> {
        if self.subject.is_none() && self.issuer.is_none() {
            return Err(XRPLModelException::ExpectedOneOf(&["subject", "issuer"]));
        }
        // When only one is provided, Account implicitly fills the other role.
        // When both are explicitly provided, rippled decides whether Account may delete
        // the credential from ledger state: issuer/subject can delete active credentials,
        // and any account can delete expired credentials. The model cannot know whether
        // the on-ledger credential is expired, so match xrpl.js and do not reject third-
        // party submitters here.
        Ok(())
    }

    fn _get_credential_type_error(&self) -> XRPLModelResult<()> {
        super::validate_credential_type(&self.credential_type)
    }
}

pub trait CredentialDeleteError {
    fn _get_subject_or_issuer_error(&self) -> XRPLModelResult<()>;
    fn _get_credential_type_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Model, XRPLModelException};
    use alloc::borrow::Cow;
    use proptest::prelude::*;

    #[test]
    fn test_requires_subject_or_issuer() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubmitter111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: None,
            issuer: None,
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_valid_with_subject() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_both_provided_allows_third_party_submitter() {
        // Match xrpl.js and the spec: when both Subject and Issuer are provided,
        // rippled may allow a third-party Account to delete an expired credential.
        // Model validation cannot know the credential's expiration state.
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubmitter111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: Some("rIssuer111111111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_with_issuer_only() {
        // When only issuer is provided, account implicitly fills the subject role.
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: None,
            issuer: Some("rIssuer111111111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_subject_only_account_is_implicit_issuer() {
        // When only subject is provided, account implicitly fills the issuer role.
        // Account does NOT need to match subject — this was the bug.
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubmitter111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_issuer_only_account_is_implicit_subject() {
        // Account does NOT need to match issuer when subject is omitted.
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubmitter111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: None,
            issuer: Some("rIssuer111111111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_both_provided_account_matches_subject() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: Some("rIssuer111111111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_both_provided_account_matches_issuer() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: Some("rIssuer111111111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubmitter111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                fee: Some("10".into()),
                sequence: Some(9),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: "4B5943".into(),
        };

        let default_json_str = r#"{"Account":"rSubmitter111111111111111111111111","TransactionType":"CredentialDelete","Fee":"10","Flags":0,"Sequence":9,"SigningPubKey":"","Subject":"rSubject11111111111111111111111111","CredentialType":"4B5943"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CredentialDelete = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_credential_type_empty_error() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: Cow::from(""),
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::ValueTooShort {
                field: "credential_type".into(),
                min: 1,
                found: 0,
            }
        );
    }

    #[test]
    fn test_credential_type_non_hex_error() {
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: "NOT_HEX".into(),
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::InvalidValueFormat {
                field: "credential_type".into(),
                format: "hexadecimal".into(),
                found: "NOT_HEX".into(),
            }
        );
    }

    #[test]
    fn test_credential_type_exceeds_128_hex_chars_error() {
        let too_long: Cow<'_, str> = Cow::from("A".repeat(129));
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: too_long,
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::ValueTooLong {
                field: "credential_type".into(),
                max: 128,
                found: 129,
            }
        );
    }

    #[test]
    fn test_credential_type_at_max_128_ok() {
        let max_hex: Cow<'_, str> = Cow::from("A".repeat(128));
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSubject11111111111111111111111111".into()),
            issuer: None,
            credential_type: max_hex,
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_self_issued_credential_delete_both_subject_and_issuer_equal_account() {
        // Self-issued credential: Account is both the subject and the issuer
        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: "rSelfIssuer1111111111111111111111".into(),
                transaction_type: TransactionType::CredentialDelete,
                ..Default::default()
            },
            subject: Some("rSelfIssuer1111111111111111111111".into()),
            issuer: Some("rSelfIssuer1111111111111111111111".into()),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }

    const ACCOUNTS: [&str; 3] = [
        "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb",
        "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de",
        "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
    ];

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn prop_subject_only_any_submitter_valid(acct_idx in 0_usize..3) {
            let tx = CredentialDelete {
                common_fields: CommonFields {
                    account: ACCOUNTS[acct_idx].into(),
                    transaction_type: TransactionType::CredentialDelete,
                    ..Default::default()
                },
                subject: Some(ACCOUNTS[1].into()),
                issuer: None,
                credential_type: "4B5943".into(),
            };
            prop_assert!(tx.get_errors().is_ok());
        }

        #[test]
        fn prop_issuer_only_any_submitter_valid(acct_idx in 0_usize..3) {
            let tx = CredentialDelete {
                common_fields: CommonFields {
                    account: ACCOUNTS[acct_idx].into(),
                    transaction_type: TransactionType::CredentialDelete,
                    ..Default::default()
                },
                subject: None,
                issuer: Some(ACCOUNTS[0].into()),
                credential_type: "4B5943".into(),
            };
            prop_assert!(tx.get_errors().is_ok());
        }

        #[test]
        fn prop_both_any_submitter_valid(acct_idx in 0_usize..3) {
            let tx = CredentialDelete {
                common_fields: CommonFields {
                    account: ACCOUNTS[acct_idx].into(),
                    transaction_type: TransactionType::CredentialDelete,
                    ..Default::default()
                },
                subject: Some(ACCOUNTS[0].into()),
                issuer: Some(ACCOUNTS[1].into()),
                credential_type: "4B5943".into(),
            };
            prop_assert!(tx.get_errors().is_ok());
        }

        #[test]
        fn prop_serde_roundtrip(
            ct in "[0-9A-F]{2,64}",
            has_subject in proptest::bool::ANY,
            has_issuer in proptest::bool::ANY,
        ) {
            let subject = if has_subject || !has_issuer { Some(Cow::Borrowed(ACCOUNTS[0])) } else { None };
            let issuer = if has_issuer { Some(Cow::Borrowed(ACCOUNTS[1])) } else { None };
            let acct = if subject.is_some() { ACCOUNTS[0] } else { ACCOUNTS[1] };
            let tx = CredentialDelete {
                common_fields: CommonFields {
                    account: acct.into(),
                    transaction_type: TransactionType::CredentialDelete,
                    fee: Some("10".into()),
                    sequence: Some(7),
                    signing_pub_key: Some(Cow::Borrowed("")),
                    ..Default::default()
                },
                subject,
                issuer,
                credential_type: Cow::Owned(ct),
            };
            prop_assert!(tx.get_errors().is_ok());
            let json = serde_json::to_string(&tx).unwrap();
            let rt: CredentialDelete = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&tx, &rt);
        }
    }
}
