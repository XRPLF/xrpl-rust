use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::constants::MAX_CREDENTIAL_URI_LENGTH;
use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, XRPLModelException, XRPLModelResult,
};
use crate::models::{FlagCollection, NoFlags};

use super::CommonTransactionBuilder;

/// A CredentialCreate transaction creates a credential object.
///
/// See CredentialCreate:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0070-credentials>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialCreate<'a> {
    /// The base fields for all transaction models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account this credential is issued to.
    pub subject: Cow<'a, str>,
    /// A hex-encoded value identifying the credential type from this issuer.
    pub credential_type: Cow<'a, str>,
    /// Optional expiration for the credential.
    pub expiration: Option<u32>,
    /// Optional additional data, represented as a hex-encoded string.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
}

impl<'a> Model for CredentialCreate<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_credential_type_error()?;
        self._get_uri_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for CredentialCreate<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CredentialCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> CredentialCreate<'a> {
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
        subject: Cow<'a, str>,
        credential_type: Cow<'a, str>,
        expiration: Option<u32>,
        uri: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CredentialCreate,
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
            credential_type,
            expiration,
            uri,
        }
    }

    pub fn with_expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_uri(mut self, uri: Cow<'a, str>) -> Self {
        self.uri = Some(uri);
        self
    }
}

impl<'a> CredentialCreateError for CredentialCreate<'a> {
    fn _get_credential_type_error(&self) -> XRPLModelResult<()> {
        super::validate_credential_type(&self.credential_type)
    }

    fn _get_uri_error(&self) -> XRPLModelResult<()> {
        if let Some(uri) = &self.uri {
            if uri.is_empty() {
                return Err(XRPLModelException::ValueTooShort {
                    field: "uri".into(),
                    min: 1,
                    found: 0,
                });
            }
            if uri.len() > MAX_CREDENTIAL_URI_LENGTH {
                return Err(XRPLModelException::ValueTooLong {
                    field: "uri".into(),
                    max: MAX_CREDENTIAL_URI_LENGTH,
                    found: uri.len(),
                });
            }
            if !uri.len().is_multiple_of(2) {
                return Err(XRPLModelException::InvalidValueFormat {
                    field: "uri".into(),
                    format: "even-length hexadecimal (whole bytes)".into(),
                    found: uri.as_ref().into(),
                });
            }
            super::validate_hex("uri", uri)?;
        }
        Ok(())
    }
}

pub trait CredentialCreateError {
    fn _get_credential_type_error(&self) -> XRPLModelResult<()>;
    fn _get_uri_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Model, XRPLModelException};
    use alloc::borrow::Cow;
    use proptest::prelude::*;

    #[test]
    fn test_serde() {
        let default_txn = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                fee: Some("10".into()),
                sequence: Some(7),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: Some(789004799),
            uri: Some("69736162656C2E636F6D2F63726564656E7469616C732F6B79632F616C696365".into()),
        };

        let default_json_str = r#"{"Account":"rIssuer111111111111111111111111111","TransactionType":"CredentialCreate","Fee":"10","Flags":0,"Sequence":7,"SigningPubKey":"","Subject":"rSubject11111111111111111111111111","CredentialType":"4B5943","Expiration":789004799,"URI":"69736162656C2E636F6D2F63726564656E7469616C732F6B79632F616C696365"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CredentialCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_credential_type_empty_error() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: Cow::from(""),
            expiration: None,
            uri: None,
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
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "NOTHEX".into(), // even-length, but 'N','O','T' are non-hex
            expiration: None,
            uri: None,
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::InvalidValueFormat {
                field: "credential_type".into(),
                format: "hexadecimal".into(),
                found: "NOTHEX".into(),
            }
        );
    }

    #[test]
    fn test_credential_type_odd_length_error() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "ABC".into(), // 3 chars — odd-length, valid hex chars
            expiration: None,
            uri: None,
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::InvalidValueFormat {
                field: "credential_type".into(),
                format: "even-length hexadecimal (whole bytes)".into(),
                found: "ABC".into(),
            }
        );
    }

    #[test]
    fn test_credential_type_at_max_128_hex_chars_ok() {
        // 128 hex chars = 64 bytes, the spec maximum
        let max_hex: Cow<'_, str> = Cow::from("A".repeat(128));
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: max_hex,
            expiration: None,
            uri: None,
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_credential_type_exceeds_128_hex_chars_error() {
        // 129 hex chars exceeds the limit
        let too_long: Cow<'_, str> = Cow::from("A".repeat(129));
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: too_long,
            expiration: None,
            uri: None,
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
    fn test_uri_at_max_256_hex_chars_ok() {
        // 256 hex chars = 128 decoded bytes; both rippled and xrpl.js cap at 256 hex chars
        let max_uri: Cow<'_, str> = Cow::from("A".repeat(MAX_CREDENTIAL_URI_LENGTH));
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: Some(max_uri),
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_uri_exceeds_256_hex_chars_error() {
        let too_long: Cow<'_, str> = Cow::from("A".repeat(MAX_CREDENTIAL_URI_LENGTH + 1));
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: Some(too_long),
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::ValueTooLong {
                field: "uri".into(),
                max: MAX_CREDENTIAL_URI_LENGTH,
                found: MAX_CREDENTIAL_URI_LENGTH + 1,
            }
        );
    }

    #[test]
    fn test_uri_empty_error() {
        // Spec section 3.2: "The URI field is empty" is a failure condition.
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: Some(Cow::from("")),
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::ValueTooShort {
                field: "uri".into(),
                min: 1,
                found: 0,
            }
        );
    }

    #[test]
    fn test_uri_odd_length_error() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: Some(Cow::from("ABC")), // 3 chars — odd-length valid hex
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::InvalidValueFormat {
                field: "uri".into(),
                format: "even-length hexadecimal (whole bytes)".into(),
                found: "ABC".into(),
            }
        );
    }

    #[test]
    fn test_uri_non_hex_error() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: Some(Cow::from("NOTHEX")), // 6 chars, even-length, non-hex
        };
        assert_eq!(
            tx.get_errors().unwrap_err(),
            XRPLModelException::InvalidValueFormat {
                field: "uri".into(),
                format: "hexadecimal".into(),
                found: "NOTHEX".into(),
            }
        );
    }

    #[test]
    fn test_subject_same_as_account_self_issued_ok() {
        // Per the spec, an issuer can issue a credential to themselves
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rSelfIssuer1111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSelfIssuer1111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: None,
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_minimal_credential_create() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "AB".into(),
            expiration: None,
            uri: None,
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_uri_none_ok() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "4B5943".into(),
            expiration: None,
            uri: None,
        };
        assert!(tx.get_errors().is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn prop_credential_type_valid_length(len in 1_usize..=64) {
            let ct = "AB".repeat(len); // even-length hex pairs — valid encodable bytes
            let tx = CredentialCreate {
                common_fields: CommonFields {
                    account: "rIssuer111111111111111111111111111".into(),
                    transaction_type: TransactionType::CredentialCreate,
                    ..Default::default()
                },
                subject: "rSubject11111111111111111111111111".into(),
                credential_type: Cow::Owned(ct),
                expiration: None,
                uri: None,
            };
            prop_assert!(tx.get_errors().is_ok(), "len {} should be valid", len);
        }

        #[test]
        fn prop_credential_type_too_long(extra in 1_usize..=100) {
            let len = 64 + extra; // "AB".repeat(64) = 128 chars (max); exceed that
            let ct = "AB".repeat(len);
            let tx = CredentialCreate {
                common_fields: CommonFields {
                    account: "rIssuer111111111111111111111111111".into(),
                    transaction_type: TransactionType::CredentialCreate,
                    ..Default::default()
                },
                subject: "rSubject11111111111111111111111111".into(),
                credential_type: Cow::Owned(ct),
                expiration: None,
                uri: None,
            };
            prop_assert!(tx.get_errors().is_err(), "len {} should be rejected", len);
        }

        #[test]
        fn prop_serde_roundtrip(
            ct in "[0-9A-F]{2,128}",
            has_expiration in proptest::bool::ANY,
            expiration_val in proptest::num::u32::ANY,
            has_uri in proptest::bool::ANY,
            uri_hex in "[0-9A-F]{2,200}",
        ) {
            let tx = CredentialCreate {
                common_fields: CommonFields {
                    account: "rIssuer111111111111111111111111111".into(),
                    transaction_type: TransactionType::CredentialCreate,
                    fee: Some("12".into()),
                    sequence: Some(42),
                    signing_pub_key: Some(Cow::Borrowed("")),
                    ..Default::default()
                },
                subject: "rSubject11111111111111111111111111".into(),
                credential_type: Cow::Owned(ct),
                expiration: if has_expiration { Some(expiration_val) } else { None },
                uri: if has_uri { Some(Cow::Owned(uri_hex)) } else { None },
            };
            let json = serde_json::to_string(&tx)
                .map_err(|e| TestCaseError::fail(format!("serialize: {e}")))?;
            let rt: CredentialCreate = serde_json::from_str(&json)
                .map_err(|e| TestCaseError::fail(format!("deserialize: {e}")))?;
            prop_assert_eq!(&tx, &rt);
        }
    }
}
