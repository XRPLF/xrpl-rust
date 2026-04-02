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

/// A CredentialAccept transaction accepts a credential issued to the sender.
///
/// See CredentialAccept:
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
pub struct CredentialAccept<'a> {
    /// The base fields for all transaction models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The issuer of the credential.
    pub issuer: Cow<'a, str>,
    /// A hex-encoded value identifying the credential type from this issuer.
    pub credential_type: Cow<'a, str>,
}

impl<'a> Model for CredentialAccept<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_credential_type_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for CredentialAccept<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for CredentialAccept<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> CredentialAccept<'a> {
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
        issuer: Cow<'a, str>,
        credential_type: Cow<'a, str>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::CredentialAccept,
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
            issuer,
            credential_type,
        }
    }
}

impl<'a> CredentialAcceptError for CredentialAccept<'a> {
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

pub trait CredentialAcceptError {
    fn _get_credential_type_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;
    use alloc::borrow::Cow;

    #[test]
    fn test_serde() {
        let default_txn = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                fee: Some("10".into()),
                sequence: Some(8),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: "4B5943".into(),
        };

        let default_json_str = r#"{"Account":"rSubject11111111111111111111111111","TransactionType":"CredentialAccept","Fee":"10","Flags":0,"Sequence":8,"SigningPubKey":"","Issuer":"rIssuer111111111111111111111111111","CredentialType":"4B5943"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: CredentialAccept = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_credential_type_length_validation() {
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: "".into(),
        };
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_credential_type_empty_error() {
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: Cow::from(""),
        };
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_credential_type_too_long_error() {
        // 129 hex chars exceeds the 128 limit
        let too_long: Cow<'_, str> = Cow::from("A".repeat(129));
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: too_long,
        };
        assert!(tx.get_errors().is_err());
    }

    #[test]
    fn test_credential_type_at_max_128_ok() {
        let max_hex: Cow<'_, str> = Cow::from("A".repeat(128));
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: max_hex,
        };
        assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn test_valid_minimal_accept() {
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: "rSubject11111111111111111111111111".into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: "rIssuer111111111111111111111111111".into(),
            credential_type: "4B5943".into(),
        };
        assert!(tx.get_errors().is_ok());
    }
}
