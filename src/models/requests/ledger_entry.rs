use alloc::{borrow::Cow, string::ToString, vec::Vec};
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    requests::RequestMethod, transactions::validate_credential_type, Model, XRPLModelException,
    XRPLModelResult,
};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// Required credential selector for credential-based DepositPreauth lookup.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct AuthorizedCredential<'a> {
    pub issuer: Cow<'a, str>,
    pub credential_type: Cow<'a, str>,
}

/// Required fields for requesting a DepositPreauth if not
/// querying by object ID.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct DepositPreauth<'a> {
    pub owner: Cow<'a, str>,
    pub authorized: Option<Cow<'a, str>>,
    pub authorized_credentials: Option<Vec<AuthorizedCredential<'a>>>,
}

impl Model for DepositPreauth<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        match (&self.authorized, &self.authorized_credentials) {
            (Some(_), None) => Ok(()),
            (None, Some(credentials)) => {
                if credentials.is_empty() {
                    return Err(XRPLModelException::ValueTooShort {
                        field: "authorized_credentials".into(),
                        min: 1,
                        found: 0,
                    });
                }
                if credentials.len() > 8 {
                    return Err(XRPLModelException::ValueTooLong {
                        field: "authorized_credentials".into(),
                        max: 8,
                        found: credentials.len(),
                    });
                }
                for (idx, credential) in credentials.iter().enumerate() {
                    validate_credential_type(&credential.credential_type)?;
                    if credentials[..idx].iter().any(|previous| {
                        previous.issuer == credential.issuer
                            && previous
                                .credential_type
                                .eq_ignore_ascii_case(&credential.credential_type)
                    }) {
                        return Err(XRPLModelException::InvalidValue {
                            field: "authorized_credentials".into(),
                            expected: "unique issuer and credential_type pairs".into(),
                            found: credential.credential_type.to_string(),
                        });
                    }
                }
                Ok(())
            }
            _ => Err(XRPLModelException::ExpectedOneOf(&[
                "authorized",
                "authorized_credentials",
            ])),
        }
    }
}

/// Required fields for requesting a Credential if not
/// querying by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct Credential<'a> {
    pub subject: Cow<'a, str>,
    pub issuer: Cow<'a, str>,
    pub credential_type: Cow<'a, str>,
}

impl<'a> Model for Credential<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_credential_type(&self.credential_type)
    }
}

/// Required fields for requesting a DirectoryNode if not
/// querying by object ID.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct Directory<'a> {
    pub dir_root: Cow<'a, str>,
    pub owner: Cow<'a, str>,
    pub sub_index: Option<u8>,
}

/// Required fields for requesting a Escrow if not querying
/// by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct Escrow<'a> {
    pub owner: Cow<'a, str>,
    pub seq: u64,
}

/// Required fields for requesting a Escrow if not querying
/// by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct Offer<'a> {
    pub account: Cow<'a, str>,
    pub seq: u64,
}

/// Required fields for requesting a Ticket, if not
/// querying by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct Ticket<'a> {
    pub owner: Cow<'a, str>,
    pub ticket_sequence: u64,
}

/// Required fields for requesting a RippleState.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct RippleState<'a> {
    pub account: Cow<'a, str>,
    pub currency: Cow<'a, str>,
}

/// Required fields for requesting an Oracle ledger entry by account + document ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct OracleIdentifier<'a> {
    /// The XRPL account that controls the Oracle object.
    pub account: Cow<'a, str>,
    /// The unique identifier of the price oracle for the account.
    #[serde(rename = "oracle_document_id")]
    pub oracle_document_id: u32,
}

/// The ledger_entry method returns a single ledger object
/// from the XRP Ledger in its raw format. See ledger formats
/// for information on the different types of objects you can
/// retrieve.
///
/// See Ledger Formats:
/// `<https://xrpl.org/ledger-data-formats.html#ledger-data-formats>`
///
/// See Ledger Entry:
/// `<https://xrpl.org/ledger_entry.html#ledger_entry>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LedgerEntry<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    pub account_root: Option<Cow<'a, str>>,
    /// If true, return the requested ledger object's contents as a
    /// hex string in the XRP Ledger's binary format. Otherwise, return
    /// data in JSON format. The default is false.
    pub binary: Option<bool>,
    pub check: Option<Cow<'a, str>>,
    pub credential: Option<Credential<'a>>,
    pub deposit_preauth: Option<DepositPreauth<'a>>,
    pub directory: Option<Directory<'a>>,
    pub escrow: Option<Escrow<'a>>,
    pub index: Option<Cow<'a, str>>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    pub offer: Option<Offer<'a>>,
    pub oracle: Option<OracleIdentifier<'a>>,
    pub payment_channel: Option<Cow<'a, str>>,
    pub ripple_state: Option<RippleState<'a>>,
    pub ticket: Option<Ticket<'a>>,
}

impl<'a: 'static> Model for LedgerEntry<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_field_error()?;
        if let Some(deposit_preauth) = &self.deposit_preauth {
            deposit_preauth.get_errors()?;
        }
        if let Some(credential) = &self.credential {
            credential.get_errors()?;
        }
        Ok(())
    }
}

impl<'a> LedgerEntryError for LedgerEntry<'a> {
    fn _get_field_error(&self) -> XRPLModelResult<()> {
        let mut signing_methods: u32 = 0;
        for method in [
            self.index.clone(),
            self.account_root.clone(),
            self.check.clone(),
        ] {
            if method.is_some() {
                signing_methods += 1
            }
        }
        if self.directory.is_some() {
            signing_methods += 1
        }
        if self.offer.is_some() {
            signing_methods += 1
        }
        if self.oracle.is_some() {
            signing_methods += 1
        }
        if self.ripple_state.is_some() {
            signing_methods += 1
        }
        if self.escrow.is_some() {
            signing_methods += 1
        }
        if self.payment_channel.is_some() {
            signing_methods += 1
        }
        if self.deposit_preauth.is_some() {
            signing_methods += 1
        }
        if self.credential.is_some() {
            signing_methods += 1
        }
        if self.ticket.is_some() {
            signing_methods += 1
        }
        if signing_methods != 1 {
            Err(XRPLModelException::ExpectedOneOf(&[
                "index",
                "account_root",
                "check",
                "directory",
                "offer",
                "oracle",
                "ripple_state",
                "escrow",
                "payment_channel",
                "deposit_preauth",
                "credential",
                "ticket",
            ]))
        } else {
            Ok(())
        }
    }
}

impl<'a> Request<'a> for LedgerEntry<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Default for LedgerEntry<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::LedgerEntry,
                id: None,
            },
            account_root: None,
            binary: None,
            check: None,
            credential: None,
            deposit_preauth: None,
            directory: None,
            escrow: None,
            index: None,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash: None,
                ledger_index: None,
            }),
            offer: None,
            payment_channel: None,
            ripple_state: None,
            ticket: None,
        }
    }
}

impl<'a> LedgerEntry<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account_root: Option<Cow<'a, str>>,
        binary: Option<bool>,
        check: Option<Cow<'a, str>>,
        credential: Option<Credential<'a>>,
        deposit_preauth: Option<DepositPreauth<'a>>,
        directory: Option<Directory<'a>>,
        escrow: Option<Escrow<'a>>,
        index: Option<Cow<'a, str>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        offer: Option<Offer<'a>>,
        oracle: Option<OracleIdentifier<'a>>,
        payment_channel: Option<Cow<'a, str>>,
        ripple_state: Option<RippleState<'a>>,
        ticket: Option<Ticket<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::LedgerEntry,
                id,
            },
            index,
            account_root,
            check,
            credential,
            payment_channel,
            deposit_preauth,
            directory,
            escrow,
            offer,
            oracle,
            ripple_state,
            ticket,
            binary,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
    }
}

pub trait LedgerEntryError {
    #[allow(clippy::result_large_err)]
    fn _get_field_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod test_ledger_entry_errors {
    use super::Offer;
    use crate::models::Model;
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    #[test]
    fn test_fields_error() {
        let ledger_entry = LedgerEntry {
            account_root: Some("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into()),
            offer: Some(Offer {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 359,
            }),
            ..Default::default()
        };
        let _expected = XRPLModelException::ExpectedOneOf(&[
            "index",
            "account_root",
            "check",
            "directory",
            "offer",
            "ripple_state",
            "escrow",
            "payment_channel",
            "deposit_preauth",
            "credential",
            "ticket",
        ]);
        assert_eq!(
            ledger_entry.validate().unwrap_err().to_string().as_str(),
            "Expected one of: index, account_root, check, directory, offer, oracle, ripple_state, escrow, payment_channel, deposit_preauth, credential, ticket"
        );
    }

    #[test]
    fn test_serde() {
        let req = LedgerEntry {
            account_root: Some("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into()),
            offer: Some(Offer {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 359,
            }),
            ..Default::default()
        };
        let serialized = serde_json::to_string(&req).unwrap();

        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_deposit_preauth_with_authorized_credentials_serde() {
        let req = LedgerEntry {
            deposit_preauth: Some(DepositPreauth {
                owner: "rOwner".into(),
                authorized: None,
                authorized_credentials: Some(vec![AuthorizedCredential {
                    issuer: "rIssuer".into(),
                    credential_type: "4B5943".into(),
                }]),
            }),
            ..Default::default()
        };

        assert!(req.validate().is_ok());
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"authorized_credentials\""));
        assert!(json.contains("\"credential_type\":\"4B5943\""));
        let deserialized: LedgerEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_deposit_preauth_rejects_duplicate_authorized_credentials() {
        let req = LedgerEntry {
            deposit_preauth: Some(DepositPreauth {
                owner: "rOwner".into(),
                authorized: None,
                authorized_credentials: Some(vec![
                    AuthorizedCredential {
                        issuer: "rIssuer".into(),
                        credential_type: "4B5943".into(),
                    },
                    AuthorizedCredential {
                        issuer: "rIssuer".into(),
                        credential_type: "4B5943".into(),
                    },
                ]),
            }),
            ..Default::default()
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_deposit_preauth_rejects_case_variant_duplicate_credentials() {
        // "4b5943" and "4B5943" decode to the same bytes — must be treated as duplicates.
        let req = LedgerEntry {
            deposit_preauth: Some(DepositPreauth {
                owner: "rOwner".into(),
                authorized: None,
                authorized_credentials: Some(vec![
                    AuthorizedCredential {
                        issuer: "rIssuer".into(),
                        credential_type: "4b5943".into(),
                    },
                    AuthorizedCredential {
                        issuer: "rIssuer".into(),
                        credential_type: "4B5943".into(),
                    },
                ]),
            }),
            ..Default::default()
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_deposit_preauth_rejects_authorized_and_credentials() {
        let req = LedgerEntry {
            deposit_preauth: Some(DepositPreauth {
                owner: "rOwner".into(),
                authorized: Some("rAuthorized".into()),
                authorized_credentials: Some(vec![AuthorizedCredential {
                    issuer: "rIssuer".into(),
                    credential_type: "4B5943".into(),
                }]),
            }),
            ..Default::default()
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_credential_selector_valid() {
        let req = LedgerEntry {
            credential: Some(Credential {
                subject: "rSubject".into(),
                issuer: "rIssuer".into(),
                credential_type: "4B5943".into(),
            }),
            ..Default::default()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_credential_selector_invalid_credential_type_error() {
        let req = LedgerEntry {
            credential: Some(Credential {
                subject: "rSubject".into(),
                issuer: "rIssuer".into(),
                credential_type: "NOT_HEX".into(),
            }),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_credential_selector_empty_credential_type_error() {
        let req = LedgerEntry {
            credential: Some(Credential {
                subject: "rSubject".into(),
                issuer: "rIssuer".into(),
                credential_type: "".into(),
            }),
            ..Default::default()
        };
        assert!(req.validate().is_err());
    }
}
