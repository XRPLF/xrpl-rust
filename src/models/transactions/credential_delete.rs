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
use crate::models::{FlagCollection, NoFlags, ValidateCurrencies};

use super::CommonTransactionBuilder;

/// A CredentialDelete transaction deletes a credential object.
///
/// See CredentialDelete:
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
pub struct CredentialDelete<'a> {
    /// The base fields for all transaction models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The subject of the credential. If omitted, Account is assumed as subject.
    pub subject: Option<Cow<'a, str>>,
    /// The issuer of the credential. If omitted, Account is assumed as issuer.
    pub issuer: Option<Cow<'a, str>>,
    /// A hex-encoded value identifying the credential type from this issuer.
    pub credential_type: Cow<'a, str>,
}

impl<'a> Model for CredentialDelete<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_subject_or_issuer_error()?;
        self._get_credential_type_error()?;
        self.validate_currencies()
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
            Err(XRPLModelException::ExpectedOneOf(&["subject", "issuer"]))
        } else if let Some(subject) = &self.subject {
            if &self.common_fields.account != subject
                && self.issuer.as_ref() != Some(&self.common_fields.account)
            {
                Err(XRPLModelException::InvalidFieldCombination {
                    field: "account",
                    other_fields: &["subject", "issuer"],
                })
            } else {
                Ok(())
            }
        } else if let Some(issuer) = &self.issuer {
            if &self.common_fields.account != issuer {
                Err(XRPLModelException::InvalidFieldCombination {
                    field: "account",
                    other_fields: &["issuer"],
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

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
}

pub trait CredentialDeleteError {
    fn _get_subject_or_issuer_error(&self) -> XRPLModelResult<()>;
    fn _get_credential_type_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;

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
    fn test_account_must_match_subject_or_issuer() {
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
        assert!(tx.get_errors().is_err());
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
}
