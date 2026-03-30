use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
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
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
