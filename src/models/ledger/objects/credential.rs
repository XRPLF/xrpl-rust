use crate::models::ledger::objects::LedgerEntryType;
use crate::models::FlagCollection;
use crate::models::Model;
use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use super::{CommonFields, LedgerObject};

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum CredentialFlag {
    /// Credential has been accepted by the subject.
    LsfAccepted = 0x00010000,
}

/// A `Credential` object is an on-ledger representation of a credential.
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0070-credentials>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Credential<'a> {
    /// The base fields for all ledger object models.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, CredentialFlag>,
    /// The account the credential is for.
    pub subject: Cow<'a, str>,
    /// The account that issued the credential.
    pub issuer: Cow<'a, str>,
    /// A hex-encoded value identifying the credential type from this issuer.
    pub credential_type: Cow<'a, str>,
    /// Optional expiration for the credential.
    pub expiration: Option<u32>,
    /// Optional additional data, represented as a hex-encoded string.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
    /// A hint indicating which page of the subject's owner directory links to this object.
    pub subject_node: Cow<'a, str>,
    /// A hint indicating which page of the issuer's owner directory links to this object.
    pub issuer_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger containing the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for Credential<'a> {}

impl<'a> LedgerObject<CredentialFlag> for Credential<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Credential<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        subject: Cow<'a, str>,
        issuer: Cow<'a, str>,
        credential_type: Cow<'a, str>,
        expiration: Option<u32>,
        uri: Option<Cow<'a, str>>,
        subject_node: Cow<'a, str>,
        issuer_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Credential,
                index,
                ledger_index,
            },
            subject,
            issuer,
            credential_type,
            expiration,
            uri,
            subject_node,
            issuer_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let credential = Credential::new(
            Some(Cow::from(
                "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001",
            )),
            None,
            Cow::from("rALICE1111111111111111111111111111"),
            Cow::from("rISABEL111111111111111111111111111"),
            Cow::from("4B5943"),
            Some(789004799),
            Some(Cow::from(
                "69736162656C2E636F6D2F63726564656E7469616C732F6B79632F616C696365",
            )),
            Cow::from("0000000000000000"),
            Cow::from("0000000000000000"),
            Cow::from("3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702"),
            8,
        );
        let serialized = serde_json::to_string(&credential).unwrap();

        let deserialized: Credential = serde_json::from_str(&serialized).unwrap();

        assert_eq!(credential, deserialized);
    }

    #[test]
    fn test_serde_round_trip_all_fields() {
        // Full credential with every field populated
        let credential = Credential {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Credential,
                index: Some(Cow::from(
                    "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001",
                )),
                ledger_index: Some(Cow::from("42")),
            },
            subject: Cow::from("rALICE1111111111111111111111111111"),
            issuer: Cow::from("rISABEL111111111111111111111111111"),
            credential_type: Cow::from("4B5943"),
            expiration: Some(789004799),
            uri: Some(Cow::from(
                "69736162656C2E636F6D2F63726564656E7469616C732F6B79632F616C696365",
            )),
            subject_node: Cow::from("0000000000000000"),
            issuer_node: Cow::from("0000000000000001"),
            previous_txn_id: Cow::from(
                "3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702",
            ),
            previous_txn_lgr_seq: 8,
        };
        let json = serde_json::to_string(&credential).unwrap();
        let deserialized: Credential = serde_json::from_str(&json).unwrap();
        assert_eq!(credential, deserialized);
    }

    #[test]
    fn test_serde_round_trip_optional_fields_omitted() {
        // Credential without optional expiration and URI
        let credential = Credential {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Credential,
                index: Some(Cow::from(
                    "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001",
                )),
                ledger_index: None,
            },
            subject: Cow::from("rALICE1111111111111111111111111111"),
            issuer: Cow::from("rISABEL111111111111111111111111111"),
            credential_type: Cow::from("4B5943"),
            expiration: None,
            uri: None,
            subject_node: Cow::from("0000000000000000"),
            issuer_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from(
                "3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702",
            ),
            previous_txn_lgr_seq: 5,
        };
        let json = serde_json::to_string(&credential).unwrap();
        // Verify optional fields are absent from serialized JSON
        assert!(!json.contains("Expiration"));
        assert!(!json.contains("URI"));

        let deserialized: Credential = serde_json::from_str(&json).unwrap();
        assert_eq!(credential, deserialized);
        assert!(deserialized.expiration.is_none());
        assert!(deserialized.uri.is_none());
    }

    #[test]
    fn test_lsf_accepted_flag_value() {
        // Verify the lsfAccepted flag has the correct value per the spec: 0x00010000
        assert_eq!(CredentialFlag::LsfAccepted as u32, 0x00010000);
    }

    #[test]
    fn test_serde_with_accepted_flag() {
        // Credential with the lsfAccepted flag set
        let mut flags = FlagCollection::default();
        flags.0.push(CredentialFlag::LsfAccepted);
        let credential = Credential {
            common_fields: CommonFields {
                flags,
                ledger_entry_type: LedgerEntryType::Credential,
                index: Some(Cow::from(
                    "DD40031C6C21164E7673A47C35513D52A6B0F1349A873EE0D188D8994CD4D001",
                )),
                ledger_index: None,
            },
            subject: Cow::from("rALICE1111111111111111111111111111"),
            issuer: Cow::from("rISABEL111111111111111111111111111"),
            credential_type: Cow::from("4B5943"),
            expiration: None,
            uri: None,
            subject_node: Cow::from("0000000000000000"),
            issuer_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from(
                "3E8964D5A86B3CD6B9ECB33310D4E073D64C865A5B866200AD2B7E29F8326702",
            ),
            previous_txn_lgr_seq: 10,
        };
        let json = serde_json::to_string(&credential).unwrap();
        let deserialized: Credential = serde_json::from_str(&json).unwrap();
        assert_eq!(credential, deserialized);
    }
}
