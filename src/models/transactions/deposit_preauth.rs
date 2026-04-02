use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::CommonTransactionBuilder;
use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    CredentialAuthorization, Model,
};
use crate::models::{
    FlagCollection, NoFlags, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};

/// A DepositPreauth transaction gives another account pre-approval
/// to deliver payments to the sender of this transaction.
///
/// See DepositPreauth:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/depositpreauth>`
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
pub struct DepositPreauth<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The XRP Ledger address of the sender to preauthorize.
    /// Mutually exclusive with `authorize_credentials`,
    /// `unauthorize`, and `unauthorize_credentials`.
    pub authorize: Option<Cow<'a, str>>,
    /// The credential(s) to preauthorize.
    pub authorize_credentials: Option<Vec<CredentialAuthorization<'a>>>,
    /// The XRP Ledger address of a sender whose preauthorization should be revoked.
    /// Mutually exclusive with `authorize`,
    /// `authorize_credentials`, and `unauthorize_credentials`.
    pub unauthorize: Option<Cow<'a, str>>,
    /// The credential(s) whose preauthorization should be revoked.
    pub unauthorize_credentials: Option<Vec<CredentialAuthorization<'a>>>,
}

impl<'a> Model for DepositPreauth<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_authorization_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for DepositPreauth<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for DepositPreauth<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> DepositPreauth<'a> {
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
        authorize: Option<Cow<'a, str>>,
        authorize_credentials: Option<Vec<CredentialAuthorization<'a>>>,
        unauthorize: Option<Cow<'a, str>>,
        unauthorize_credentials: Option<Vec<CredentialAuthorization<'a>>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::DepositPreauth,
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
            authorize,
            authorize_credentials,
            unauthorize,
            unauthorize_credentials,
        }
    }

    pub fn with_authorize(mut self, authorize: Cow<'a, str>) -> Self {
        self.authorize = Some(authorize);
        self
    }

    pub fn with_authorize_credentials(
        mut self,
        authorize_credentials: Vec<CredentialAuthorization<'a>>,
    ) -> Self {
        self.authorize_credentials = Some(authorize_credentials);
        self
    }

    pub fn with_unauthorize(mut self, unauthorize: Cow<'a, str>) -> Self {
        self.unauthorize = Some(unauthorize);
        self
    }

    pub fn with_unauthorize_credentials(
        mut self,
        unauthorize_credentials: Vec<CredentialAuthorization<'a>>,
    ) -> Self {
        self.unauthorize_credentials = Some(unauthorize_credentials);
        self
    }
}

impl<'a> DepositPreauthError for DepositPreauth<'a> {
    fn _get_authorization_error(&self) -> XRPLModelResult<()> {
        fn validate_credential_list(
            credentials: &[CredentialAuthorization<'_>],
            field: &'static str,
        ) -> XRPLModelResult<()> {
            let len = credentials.len();
            if credentials.is_empty() {
                return Err(XRPLModelException::ValueTooShort {
                    field: field.into(),
                    min: 1,
                    found: len,
                });
            }
            if len > 8 {
                return Err(XRPLModelException::ValueTooLong {
                    field: field.into(),
                    max: 8,
                    found: len,
                });
            }
            for (i, cred) in credentials.iter().enumerate() {
                if credentials[..i].contains(cred) {
                    return Err(XRPLModelException::ValueEqualsValue {
                        field1: field.into(),
                        field2: alloc::format!("{field} (duplicate entry)"),
                    });
                }
            }
            Ok(())
        }

        let count = [
            self.authorize.is_some(),
            self.unauthorize.is_some(),
            self.authorize_credentials.is_some(),
            self.unauthorize_credentials.is_some(),
        ]
        .iter()
        .filter(|x| **x)
        .count();

        if count != 1 {
            return Err(XRPLModelException::InvalidFieldCombination {
                field: "authorize",
                other_fields: &[
                    "unauthorize",
                    "authorize_credentials",
                    "unauthorize_credentials",
                ],
            });
        }

        if let Some(credentials) = &self.authorize_credentials {
            validate_credential_list(credentials, "authorize_credentials")?;
        }
        if let Some(credentials) = &self.unauthorize_credentials {
            validate_credential_list(credentials, "unauthorize_credentials")?;
        }

        Ok(())
    }
}

pub trait DepositPreauthError {
    fn _get_authorization_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CredentialAuthorizationFields;
    use crate::models::Model;
    use alloc::vec;

    #[test]
    fn test_authorize_and_unauthorize_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: None,
        };

        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_both_authorize_and_unauthorize_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into()),
            authorize_credentials: None,
            unauthorize: Some("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into()),
            unauthorize_credentials: None,
        };

        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_valid_with_authorize() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into()),
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: None,
        };

        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_valid_with_unauthorize() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: Some("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into()),
            unauthorize_credentials: None,
        };

        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = DepositPreauth {
            common_fields: CommonFields {
                account: "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8".into(),
                transaction_type: TransactionType::DepositPreauth,
                fee: Some("10".into()),
                sequence: Some(2),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            authorize: Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into()),
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: None,
        };

        let default_json_str = r#"{"Account":"rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8","TransactionType":"DepositPreauth","Fee":"10","Flags":0,"Sequence":2,"SigningPubKey":"","Authorize":"rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: DepositPreauth = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_authorize("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into())
        .with_fee("10".into())
        .with_sequence(123)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345);

        assert_eq!(
            deposit_preauth.authorize.as_ref().unwrap(),
            "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de"
        );
        assert!(deposit_preauth.unauthorize.is_none());
        assert_eq!(deposit_preauth.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(deposit_preauth.common_fields.sequence, Some(123));
        assert_eq!(
            deposit_preauth.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(deposit_preauth.common_fields.source_tag, Some(12345));
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_builder_with_unauthorize() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_unauthorize("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into())
        .with_fee("10".into())
        .with_sequence(123);

        assert!(deposit_preauth.authorize.is_none());
        assert_eq!(
            deposit_preauth.unauthorize.as_ref().unwrap(),
            "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"
        );
        assert_eq!(deposit_preauth.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(deposit_preauth.common_fields.sequence, Some(123));
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_valid_with_authorize_credentials() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
            unauthorize: None,
            unauthorize_credentials: None,
        };

        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_authorize_credentials_array_size_validation() {
        let mut deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(vec![]),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());

        deposit_preauth.authorize_credentials = Some(
            (0..9)
                .map(|_| {
                    CredentialAuthorization::new(crate::models::CredentialAuthorizationFields::new(
                        "rIssuer111111111111111111111111111".into(),
                        "4B5943".into(),
                    ))
                })
                .collect(),
        );
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_all_four_fields_set_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into()),
            authorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
            unauthorize: Some("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into()),
            unauthorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_no_fields_set_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_authorize_credentials_zero_entries_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(vec![]),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_authorize_credentials_nine_entries_error() {
        let creds: Vec<CredentialAuthorization<'_>> = (0..9)
            .map(|_| {
                CredentialAuthorization::new(crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ))
            })
            .collect();
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(creds),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_authorize_credentials_exactly_eight_ok() {
        let creds: Vec<CredentialAuthorization<'_>> = (0..8)
            .map(|i| {
                CredentialAuthorization::new(crate::models::CredentialAuthorizationFields::new(
                    alloc::format!("rIssuer{i}1111111111111111111111111").into(),
                    "4B5943".into(),
                ))
            })
            .collect();
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(creds),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_authorize_credentials_exactly_one_ok() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_unauthorize_credentials_zero_entries_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: Some(vec![]),
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_unauthorize_credentials_nine_entries_error() {
        let creds: Vec<CredentialAuthorization<'_>> = (0..9)
            .map(|_| {
                CredentialAuthorization::new(crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ))
            })
            .collect();
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: Some(creds),
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_unauthorize_credentials_exactly_eight_ok() {
        let creds: Vec<CredentialAuthorization<'_>> = (0..8)
            .map(|i| {
                CredentialAuthorization::new(crate::models::CredentialAuthorizationFields::new(
                    alloc::format!("rIssuer{i}1111111111111111111111111").into(),
                    "4B5943".into(),
                ))
            })
            .collect();
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: Some(creds),
        };
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_unauthorize_credentials_exactly_one_ok() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
        };
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_valid_with_unauthorize_credentials() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: None,
            unauthorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
        };
        assert!(deposit_preauth.get_errors().is_ok());
    }

    #[test]
    fn test_authorize_and_authorize_credentials_error() {
        // Setting both authorize and authorize_credentials is invalid
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: Some("rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into()),
            authorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
            unauthorize: None,
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_unauthorize_and_unauthorize_credentials_error() {
        // Setting both unauthorize and unauthorize_credentials is invalid
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            authorize_credentials: None,
            unauthorize: Some("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".into()),
            unauthorize_credentials: Some(vec![CredentialAuthorization::new(
                crate::models::CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                ),
            )]),
        };
        assert!(deposit_preauth.get_errors().is_err());
    }

    #[test]
    fn test_authorize_credentials_duplicate_entries_error() {
        let deposit_preauth = DepositPreauth {
            common_fields: CommonFields {
                account: "rOwner1111111111111111111111111111".into(),
                transaction_type: TransactionType::DepositPreauth,
                ..Default::default()
            },
            authorize: None,
            unauthorize: None,
            authorize_credentials: Some(vec![
                CredentialAuthorization::new(CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                )),
                CredentialAuthorization::new(CredentialAuthorizationFields::new(
                    "rIssuer111111111111111111111111111".into(),
                    "4B5943".into(),
                )),
            ]),
            unauthorize_credentials: None,
        };
        assert!(deposit_preauth.get_errors().is_err());
    }
}
