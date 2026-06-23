use alloc::borrow::Cow;
use alloc::string::ToString;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::core::addresscodec::is_valid_classic_address;
use crate::models::{requests::RequestMethod, Model, XRPLModelException, XRPLModelResult};
use crate::models::transactions::vault_common::validate_vault_id;

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// Required fields for requesting a DepositPreauth if not
/// querying by object ID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct DepositPreauth<'a> {
    pub authorized: Cow<'a, str>,
    pub owner: Cow<'a, str>,
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

/// Vault selector for a `ledger_entry` request (XLS-65 SingleAssetVault).
///
/// rippled accepts either a direct 256-bit hash object ID or an object
/// containing the vault owner account and the sequence number of the
/// `VaultCreate` transaction (LedgerEntry.cpp:751-764).
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum VaultIdentifier<'a> {
    /// Look up by ledger object ID (64 hex chars, nonzero).
    Id(Cow<'a, str>),
    /// Look up by vault owner account + VaultCreate sequence number.
    OwnerSeq { owner: Cow<'a, str>, seq: u32 },
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
    /// Vault selector: either a 256-bit hash ID or an owner + seq pair (XLS-65).
    pub vault: Option<VaultIdentifier<'a>>,
}

impl<'a: 'static> Model for LedgerEntry<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_field_error()?;
        if let Some(vault) = &self.vault {
            match vault {
                VaultIdentifier::Id(id) => validate_vault_id(id)?,
                VaultIdentifier::OwnerSeq { owner, seq } => {
                    if !is_valid_classic_address(owner) {
                        return Err(XRPLModelException::InvalidValue {
                            field: "vault.owner".into(),
                            expected: "a valid classic account address".into(),
                            found: owner.as_ref().into(),
                        });
                    }
                    if *seq == 0 {
                        return Err(XRPLModelException::InvalidValue {
                            field: "vault.seq".into(),
                            expected: "a positive sequence number (> 0)".into(),
                            found: seq.to_string(),
                        });
                    }
                }
            }
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
        if self.ticket.is_some() {
            signing_methods += 1
        }
        if self.vault.is_some() {
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
                "ticket",
                "vault",
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

impl<'a> LedgerEntry<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account_root: Option<Cow<'a, str>>,
        binary: Option<bool>,
        check: Option<Cow<'a, str>>,
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
        vault: Option<VaultIdentifier<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::LedgerEntry,
                id,
            },
            index,
            account_root,
            check,
            payment_channel,
            deposit_preauth,
            directory,
            escrow,
            offer,
            oracle,
            ripple_state,
            ticket,
            vault,
            binary,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
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
            deposit_preauth: None,
            directory: None,
            escrow: None,
            index: None,
            ledger_lookup: None,
            offer: None,
            oracle: None,
            payment_channel: None,
            ripple_state: None,
            ticket: None,
            vault: None,
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

    use super::*;

    #[test]
    fn test_fields_error() {
        let ledger_entry = LedgerEntry::new(
            None,
            Some("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Offer {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 359,
            }),
            None, // oracle
            None,
            None,
            None,
            None, // vault
        );
        let _expected = XRPLModelException::ExpectedOneOf(&[
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
            "ticket",
            "vault",
        ]);
        assert_eq!(
            ledger_entry.validate().unwrap_err().to_string().as_str(),
            "Expected one of: index, account_root, check, directory, offer, oracle, ripple_state, escrow, payment_channel, deposit_preauth, ticket, vault"
        );
    }

    #[test]
    fn test_vault_selector_by_id() {
        let id = "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2";
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::Id(id.into())),
            ..Default::default()
        };
        assert!(req.validate().is_ok(), "vault id selector should be valid");
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.validate().is_ok());
        assert!(serialized.contains("\"vault\""), "expected vault key in JSON");
        assert!(serialized.contains(id), "expected vault id in JSON");
    }

    #[test]
    fn test_vault_selector_by_owner_seq() {
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::OwnerSeq {
                owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 7,
            }),
            ..Default::default()
        };
        assert!(req.validate().is_ok(), "vault owner+seq selector should be valid");
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.validate().is_ok());
        assert!(serialized.contains("\"owner\""), "expected owner in JSON");
        assert!(serialized.contains("\"seq\""), "expected seq in JSON");
    }

    #[test]
    fn test_vault_id_wrong_length_rejected() {
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::Id("DEADBEEF".into())),
            ..Default::default()
        };
        assert!(req.validate().is_err(), "short vault_id must be rejected");
    }

    #[test]
    fn test_vault_id_nonhex_rejected() {
        let non_hex: alloc::string::String = "Z".repeat(64);
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::Id(non_hex.into())),
            ..Default::default()
        };
        assert!(req.validate().is_err(), "non-hex vault_id must be rejected");
    }

    #[test]
    fn test_vault_id_all_zero_rejected() {
        let zeros: alloc::string::String = "0".repeat(64);
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::Id(zeros.into())),
            ..Default::default()
        };
        assert!(req.validate().is_err(), "all-zero vault_id must be rejected");
    }

    #[test]
    fn test_vault_owner_seq_invalid_owner_rejected() {
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::OwnerSeq {
                owner: "notanaddress".into(),
                seq: 1,
            }),
            ..Default::default()
        };
        assert!(req.validate().is_err(), "invalid owner must be rejected");
    }

    #[test]
    fn test_vault_owner_seq_zero_seq_rejected() {
        let req = LedgerEntry {
            vault: Some(VaultIdentifier::OwnerSeq {
                owner: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 0,
            }),
            ..Default::default()
        };
        assert!(req.validate().is_err(), "seq == 0 must be rejected");
    }

    #[test]
    fn test_serde() {
        let req = LedgerEntry::new(
            None,
            Some("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Offer {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                seq: 359,
            }),
            None, // oracle
            None,
            None,
            None,
            None, // vault
        );
        let serialized = serde_json::to_string(&req).unwrap();

        let deserialized: LedgerEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(req, deserialized);
    }
}
