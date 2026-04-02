use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::constants::MAX_URI_LENGTH;
use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, XRPLModelException, XRPLModelResult,
};
use crate::models::{FlagCollection, NoFlags, ValidateCurrencies};

use super::CommonTransactionBuilder;

/// A CredentialCreate transaction creates a credential object.
///
/// See CredentialCreate:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0070-credentials>`
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
        self._get_uri_error()?;
        self.validate_currencies()
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
        let len = self.credential_type.len();
        if len == 0 {
            Err(XRPLModelException::ValueTooShort {
                field: "credential_type".into(),
                min: 1,
                found: 0,
            })
        } else if len > 128 {
            Err(XRPLModelException::ValueTooLong {
                field: "credential_type".into(),
                max: 128,
                found: len,
            })
        } else {
            Ok(())
        }
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
            if uri.len() > MAX_URI_LENGTH {
                return Err(XRPLModelException::ValueTooLong {
                    field: "uri".into(),
                    max: MAX_URI_LENGTH,
                    found: uri.len(),
                });
            }
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
    use crate::models::Model;
    use alloc::borrow::Cow;

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
    fn test_credential_type_length_validation() {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: "rIssuer111111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: "rSubject11111111111111111111111111".into(),
            credential_type: "".into(),
            expiration: None,
            uri: None,
        };
        assert!(tx.get_errors().is_err());
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
        assert!(tx.get_errors().is_err());
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
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_uri_at_max_length_ok() {
        // MAX_URI_LENGTH is 512 chars
        let max_uri: Cow<'_, str> = Cow::from("A".repeat(MAX_URI_LENGTH));
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
    fn test_uri_exceeds_max_length_error() {
        let too_long: Cow<'_, str> = Cow::from("A".repeat(MAX_URI_LENGTH + 1));
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
        assert!(tx.get_errors().is_err());
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
        assert!(tx.get_errors().is_err());
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
}
