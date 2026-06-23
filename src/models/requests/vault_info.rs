use alloc::borrow::Cow;
use alloc::string::ToString;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::core::addresscodec::is_valid_classic_address;
use crate::models::transactions::vault_common::validate_vault_id;
use crate::models::{requests::RequestMethod, Model, XRPLModelException, XRPLModelResult};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// Request parameters for the `vault_info` method (XLS-65 SingleAssetVault).
///
/// Exactly one lookup mode must be supplied:
/// - `vault_id` — look up by ledger object ID.
/// - `owner` + `seq` — look up by vault owner account and the `Sequence` number of
///   the `VaultCreate` transaction. Both fields are required; `seq` must be > 0.
///
/// Validated against rippled `parseVault` (VaultInfo.cpp:21-58).
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VaultInfo<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Look up by ledger object ID.
    pub vault_id: Option<Cow<'a, str>>,
    /// Look up by vault owner account address.
    pub owner: Option<Cow<'a, str>>,
    /// The sequence number of the `VaultCreate` transaction; used together
    /// with `owner` to identify the vault. Must be > 0.
    pub seq: Option<u32>,
    /// The unique identifier of the ledger version to use.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
}

impl<'a> Model for VaultInfo<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        let has_id = self.vault_id.is_some();
        let has_owner = self.owner.is_some();
        let has_seq = self.seq.is_some();

        if has_id && !has_owner && !has_seq {
            validate_vault_id(self.vault_id.as_deref().unwrap())?;
            Ok(())
        } else if !has_id && has_owner && has_seq {
            // owner + seq lookup: validate both fields match rippled's checks.
            let owner = self.owner.as_deref().unwrap();
            if !is_valid_classic_address(owner) {
                return Err(XRPLModelException::InvalidValue {
                    field: "owner".into(),
                    expected: "a valid classic account address".into(),
                    found: owner.into(),
                });
            }
            let seq = self.seq.unwrap();
            if seq == 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "seq".into(),
                    expected: "a positive sequence number (> 0)".into(),
                    found: seq.to_string(),
                });
            }
            Ok(())
        } else {
            // Anything else: neither, both, or owner-without-seq / seq-without-owner.
            Err(XRPLModelException::ExpectedOneOf(&[
                "vault_id",
                "owner (with seq)",
            ]))
        }
    }
}

impl<'a> Request<'a> for VaultInfo<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> VaultInfo<'a> {
    /// Construct a `vault_info` request using a ledger object ID lookup.
    pub fn new(
        id: Option<Cow<'a, str>>,
        vault_id: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id,
            },
            vault_id: Some(vault_id),
            owner: None,
            seq: None,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
    }

    /// Construct a `vault_info` request using an owner + sequence lookup.
    ///
    /// `seq` must be the `Sequence` of the `VaultCreate` transaction and must be > 0.
    pub fn new_by_owner(
        id: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        seq: u32,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id,
            },
            vault_id: None,
            owner: Some(owner),
            seq: Some(seq),
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::testing::test_constants::*;

    const VAULT_HEX_ID: &str = "AABBCCDDEEFF00112233445566778899AABBCCDDEEFF00112233445566778899";

    // --- vault_id lookup ---

    #[test]
    fn test_vault_info_new() {
        let req = VaultInfo::new(None, VAULT_HEX_ID.into(), None, None);
        assert_eq!(req.vault_id.as_deref(), Some(VAULT_HEX_ID));
        assert_eq!(req.common_fields.command, RequestMethod::VaultInfo);
        assert!(req.owner.is_none());
        assert!(req.seq.is_none());
    }

    #[test]
    fn test_vault_id_lookup_valid() {
        let req = VaultInfo::new(None, VAULT_HEX_ID.into(), None, None);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_vault_info_serde() {
        let req = VaultInfo::new(Some("req-1".into()), VAULT_HEX_ID.into(), None, None);
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: VaultInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(
            serialized.contains("\"vault_id\""),
            "expected vault_id key: {serialized}"
        );
        assert!(
            serialized.contains("\"vault_info\""),
            "expected command vault_info: {serialized}"
        );
    }

    // --- owner + seq lookup ---

    #[test]
    fn test_vault_info_new_by_owner() {
        let req = VaultInfo::new_by_owner(None, ACCOUNT_HOLDER.into(), 5, None, None);
        assert!(req.vault_id.is_none());
        assert_eq!(req.owner.as_deref(), Some(ACCOUNT_HOLDER));
        assert_eq!(req.seq, Some(5));
        assert_eq!(req.common_fields.command, RequestMethod::VaultInfo);
    }

    #[test]
    fn test_owner_seq_lookup_valid() {
        let req = VaultInfo::new_by_owner(None, ACCOUNT_HOLDER.into(), 5, None, None);
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_owner_seq_serde() {
        let req = VaultInfo::new_by_owner(None, ACCOUNT_HOLDER.into(), 7, None, None);
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: VaultInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(
            serialized.contains("\"owner\""),
            "expected owner key: {serialized}"
        );
        assert!(
            serialized.contains("\"seq\""),
            "expected seq key: {serialized}"
        );
        assert!(
            !serialized.contains("\"vault_id\""),
            "vault_id must be absent: {serialized}"
        );
    }

    // --- validation error cases ---

    #[test]
    fn test_neither_selector_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: None,
            owner: None,
            seq: None,
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_both_selectors_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: Some(VAULT_HEX_ID.into()),
            owner: Some(ACCOUNT_HOLDER.into()),
            seq: Some(5),
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_owner_without_seq_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: None,
            owner: Some(ACCOUNT_HOLDER.into()),
            seq: None,
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_seq_without_owner_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: None,
            owner: None,
            seq: Some(5),
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_seq_zero_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: None,
            owner: Some(ACCOUNT_HOLDER.into()),
            seq: Some(0),
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_invalid_owner_address_rejected() {
        let req = VaultInfo {
            common_fields: CommonFields {
                command: RequestMethod::VaultInfo,
                id: None,
            },
            vault_id: None,
            owner: Some("notanaddress".into()),
            seq: Some(5),
            ledger_lookup: None,
        };
        assert!(req.validate().is_err());
    }

    // --- vault_id content validation ---

    #[test]
    fn test_vault_id_wrong_length_rejected() {
        let req = VaultInfo::new(None, "DEADBEEF".into(), None, None);
        assert!(req.validate().is_err(), "short vault_id must be rejected");
    }

    #[test]
    fn test_vault_id_nonhex_rejected() {
        let non_hex: alloc::string::String = "Z".repeat(64);
        let req = VaultInfo::new(None, non_hex.into(), None, None);
        assert!(req.validate().is_err(), "non-hex vault_id must be rejected");
    }

    #[test]
    fn test_vault_id_all_zero_rejected() {
        let zeros: alloc::string::String = "0".repeat(64);
        let req = VaultInfo::new(None, zeros.into(), None, None);
        assert!(
            req.validate().is_err(),
            "all-zero vault_id must be rejected"
        );
    }
}
