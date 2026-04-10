use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, XRPLModelResult,
};
use crate::models::{FlagCollection, NoFlags};

use super::{
    exceptions::{XRPLDIDSetException, XRPLTransactionException},
    CommonFields, CommonTransactionBuilder,
};

/// Maximum length in hex characters for DID fields (Data, DIDDocument, URI).
/// Each field is limited to 256 bytes, which is 256 hex characters in
/// the client library convention (matching xrpl-py).
pub const MAX_DID_FIELD_LENGTH: usize = 256;

/// Create or update a DID (Decentralized Identifier) associated with
/// the sending account.
///
/// See DIDSet:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/didset>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DIDSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The public attestations of identity credentials associated with the DID.
    pub data: Option<Cow<'a, str>>,
    /// The DID document associated with the DID.
    #[serde(rename = "DIDDocument")]
    pub did_document: Option<Cow<'a, str>>,
    /// The Universal Resource Identifier associated with the DID.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
}

impl<'a> Model for DIDSet<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.get_did_field_errors()
    }
}

impl<'a> Transaction<'a, NoFlags> for DIDSet<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for DIDSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

/// Returns true if the string is valid hexadecimal (only chars 0-9, a-f, A-F).
/// An empty string is considered valid hex.
fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

impl<'a> DIDSet<'a> {
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
        data: Option<Cow<'a, str>>,
        did_document: Option<Cow<'a, str>>,
        uri: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::DIDSet,
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
            data,
            did_document,
            uri,
        }
    }

    /// Validate the DID-specific fields.
    fn get_did_field_errors(&self) -> XRPLModelResult<()> {
        // At least one of data, did_document, uri must be provided
        if self.data.is_none() && self.did_document.is_none() && self.uri.is_none() {
            return Err(XRPLTransactionException::from(
                XRPLDIDSetException::MustHaveAtLeastOneField,
            )
            .into());
        }

        // If all provided fields are empty strings, that's invalid
        let all_empty = self.data.as_deref().is_none_or(|s| s.is_empty())
            && self.did_document.as_deref().is_none_or(|s| s.is_empty())
            && self.uri.as_deref().is_none_or(|s| s.is_empty());

        // Only check "all empty" if at least one field IS provided (we already checked all-None above)
        if all_empty {
            return Err(XRPLTransactionException::from(
                XRPLDIDSetException::AtLeastOneFieldMustBeNonEmpty,
            )
            .into());
        }

        // Validate each field individually
        self.validate_did_field("data", self.data.as_deref())?;
        self.validate_did_field("did_document", self.did_document.as_deref())?;
        self.validate_did_field("uri", self.uri.as_deref())?;

        Ok(())
    }

    /// Validate a single DID field for hex format and max length.
    fn validate_did_field(&self, field_name: &str, value: Option<&str>) -> XRPLModelResult<()> {
        if let Some(val) = value {
            if val.is_empty() {
                // Empty string is valid (used to delete a field)
                return Ok(());
            }

            let valid_hex = is_hex(val);
            let valid_length = val.len() <= MAX_DID_FIELD_LENGTH;

            if !valid_hex && !valid_length {
                return Err(XRPLTransactionException::from(
                    XRPLDIDSetException::InvalidFieldHexAndTooLong {
                        field: field_name.into(),
                        found_length: val.len(),
                    },
                )
                .into());
            }

            if !valid_hex {
                return Err(
                    XRPLTransactionException::from(XRPLDIDSetException::InvalidFieldHex {
                        field: field_name.into(),
                    })
                    .into(),
                );
            }

            if !valid_length {
                return Err(
                    XRPLTransactionException::from(XRPLDIDSetException::FieldTooLong {
                        field: field_name.into(),
                        max: MAX_DID_FIELD_LENGTH,
                        found: val.len(),
                    })
                    .into(),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = "r9LqNeG6qHxjeUocjvVki2XR35weJ9mZgQ";
    const VALID_FIELD: &str = "1234567890abcdefABCDEF";
    const TOO_LONG_FIELD: &str = concat!(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "A"
    ); // 257 chars
    const BAD_HEX_FIELD: &str = "random_non_hex_content";

    #[test]
    fn test_valid_all_fields() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some(VALID_FIELD.into()),
            did_document: Some(VALID_FIELD.into()),
            uri: Some(VALID_FIELD.into()),
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_valid_only_data() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some(VALID_FIELD.into()),
            did_document: None,
            uri: None,
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_valid_only_did_document() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: None,
            did_document: Some(VALID_FIELD.into()),
            uri: None,
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_valid_only_uri() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: None,
            did_document: None,
            uri: Some(VALID_FIELD.into()),
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_empty_no_fields() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: None,
            did_document: None,
            uri: None,
        };
        assert!(!tx.is_valid());
        let err = tx.validate().unwrap_err();
        assert!(err.to_string().contains("Must have at least one"));
    }

    #[test]
    fn test_all_empty_strings() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some("".into()),
            did_document: Some("".into()),
            uri: Some("".into()),
        };
        assert!(!tx.is_valid());
        let err = tx.validate().unwrap_err();
        assert!(err
            .to_string()
            .contains("must have a length greater than zero"));
    }

    #[test]
    fn test_single_empty_data_field_is_valid() {
        // An empty string for one field is valid (used to delete that field)
        // as long as at least one other field is non-empty or not all are empty
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some("".into()),
            did_document: Some(VALID_FIELD.into()),
            uri: None,
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_too_long() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: None,
            did_document: Some(TOO_LONG_FIELD.into()),
            uri: None,
        };
        assert!(!tx.is_valid());
        let err = tx.validate().unwrap_err();
        assert!(err.to_string().contains("256"));
    }

    #[test]
    fn test_not_hex() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some(BAD_HEX_FIELD.into()),
            did_document: None,
            uri: None,
        };
        assert!(!tx.is_valid());
        let err = tx.validate().unwrap_err();
        assert!(err.to_string().contains("hex"));
    }

    #[test]
    fn test_too_long_and_not_hex() {
        // 257 non-hex chars
        let bad_field: alloc::string::String = "q".repeat(257);
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: None,
            did_document: None,
            uri: Some(bad_field.into()),
        };
        assert!(!tx.is_valid());
        let err = tx.validate().unwrap_err();
        assert!(err.to_string().contains("hex"));
        assert!(err.to_string().contains("256"));
    }

    #[test]
    fn test_serialize() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::DIDSet,
                fee: Some("10".into()),
                sequence: Some(391),
                signing_pub_key: Some(
                    "0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020".into(),
                ),
                ..Default::default()
            },
            data: Some("617474657374".into()),
            did_document: Some("646F63".into()),
            uri: Some("6469645F6578616D706C65".into()),
        };

        let expected_json = r#"{"Account":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh","TransactionType":"DIDSet","Fee":"10","Flags":0,"Sequence":391,"SigningPubKey":"0330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD020","Data":"617474657374","DIDDocument":"646F63","URI":"6469645F6578616D706C65"}"#;

        let serialized = serde_json::to_string(&tx).unwrap();
        let expected_value = serde_json::to_value(expected_json).unwrap();
        let serialized_value = serde_json::to_value(&serialized).unwrap();
        assert_eq!(serialized_value, expected_value);

        let deserialized: DIDSet = serde_json::from_str(expected_json).unwrap();
        assert_eq!(tx, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some("617474657374".into()),
            did_document: Some("646F63".into()),
            uri: Some("6469645F6578616D706C65".into()),
        }
        .with_fee("10".into())
        .with_sequence(391)
        .with_last_ledger_sequence(7108682);

        assert_eq!(tx.data.as_deref(), Some("617474657374"));
        assert_eq!(tx.did_document.as_deref(), Some("646F63"));
        assert_eq!(tx.uri.as_deref(), Some("6469645F6578616D706C65"));
        assert_eq!(tx.common_fields.fee.as_ref().unwrap().0, "10");
        assert_eq!(tx.common_fields.sequence, Some(391));
        assert_eq!(tx.common_fields.last_ledger_sequence, Some(7108682));
    }

    #[test]
    fn test_default() {
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some(VALID_FIELD.into()),
            ..Default::default()
        };

        assert_eq!(tx.common_fields.account, ACCOUNT);
        assert_eq!(tx.common_fields.transaction_type, TransactionType::DIDSet);
        assert_eq!(tx.data.as_deref(), Some(VALID_FIELD));
        assert!(tx.did_document.is_none());
        assert!(tx.uri.is_none());
    }

    #[test]
    fn test_max_length_exactly_256() {
        let max_field: alloc::string::String = "A".repeat(256);
        let tx = DIDSet {
            common_fields: CommonFields {
                account: ACCOUNT.into(),
                transaction_type: TransactionType::DIDSet,
                ..Default::default()
            },
            data: Some(max_field.into()),
            did_document: None,
            uri: None,
        };
        assert!(tx.is_valid());
    }
}
