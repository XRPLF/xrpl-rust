use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::ledger::objects::vault::Vault;

/// The share token (`MPTokenIssuance`) embedded in a `vault_info` response.
///
/// xrpld nests the vault's share `MPTokenIssuance` inside the `vault` object
/// under the lowercase `"shares"` key.  All fields are `Option` to tolerate
/// server omissions; keys are PascalCase as returned by xrpld.
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0033d>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct VaultShares<'a> {
    /// Always `"MPTokenIssuance"` for this object.
    pub ledger_entry_type: Option<Cow<'a, str>>,
    pub flags: Option<u32>,
    /// Account address of the MPTokenIssuance issuer (the vault pseudo-account).
    pub issuer: Option<Cow<'a, str>>,
    /// Total amount outstanding (string integer).
    pub outstanding_amount: Option<Cow<'a, str>>,
    /// Maximum amount that may ever be issued; absent means no cap.
    pub maximum_amount: Option<Cow<'a, str>>,
    /// Scale — power of ten multiplier for the asset value.
    pub asset_scale: Option<u8>,
    /// Transfer fee in basis-point units (0–50 000).
    pub transfer_fee: Option<u16>,
    /// Arbitrary metadata about the share token (hex-encoded).
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
    /// Sequence number of the transaction that created this issuance.
    pub sequence: Option<u32>,
    pub owner_node: Option<Cow<'a, str>>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Option<Cow<'a, str>>,
    pub previous_txn_lgr_seq: Option<u32>,
    /// Computed ID of this `MPTokenIssuance` (48 hex chars); matches `Vault.ShareMPTID`.
    /// xrpld includes this field on `MPTokenIssuance` objects in `account_objects` responses
    /// and in the embedded `shares` subobject of `vault_info`.
    #[serde(rename = "mpt_issuance_id")]
    pub mpt_issuance_id: Option<Cow<'a, str>>,
}

/// The vault object as returned by `vault_info` (XLS-65 SingleAssetVault).
///
/// Wraps all on-ledger [`Vault`] fields (via `#[serde(flatten)]`) plus the
/// `shares` sub-object that `vault_info` appends, containing the vault's
/// share [`MPTokenIssuance`](VaultShares) data.
///
/// The flat `#[serde(flatten)]` means this struct serialises/deserialises
/// identically to the raw `Vault` ledger object extended with a `"shares"`
/// key — no extra nesting.
///
/// # Flatten chain
///
/// `VaultObject` flattens `Vault<'a>`, which itself flattens `CommonFields`.
/// This two-level chain is safe for JSON, but `#[serde(deny_unknown_fields)]`
/// must **not** be added to any struct in this chain — serde does not propagate
/// it through flattened structs and silently drops unknown fields instead.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultObject<'a> {
    /// All on-ledger `Vault` fields (Owner, Account, Asset, AssetsTotal, etc.).
    #[serde(flatten)]
    pub vault: Vault<'a>,
    /// The share `MPTokenIssuance` appended by the `vault_info` RPC.
    /// Present on every successful `vault_info` response; absent in raw
    /// ledger-entry data.
    pub shares: Option<VaultShares<'a>>,
}

/// Response from the `vault_info` method (XLS-65 SingleAssetVault).
///
/// xrpld returns the vault object under a `"vault"` key (not `"node"`).
/// The nested `"shares"` sub-object within `"vault"` is the vault's share
/// `MPTokenIssuance`, typed as [`VaultShares`].
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VaultInfo<'a> {
    /// The vault object with all on-ledger fields plus the share token.
    pub vault: Option<VaultObject<'a>>,
    /// The ledger sequence number current at request time (open-ledger mode).
    pub ledger_current_index: Option<u32>,
    /// The ledger index of the validated ledger used to retrieve this data.
    pub ledger_index: Option<u32>,
    /// Identifying hash of the ledger version used to retrieve this data.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// Whether this data is from a validated ledger version.
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Currency;

    /// Minimal valid vault JSON (XRP asset, all Vault required fields present).
    const VAULT_JSON: &str = r#"{
        "vault": {
            "LedgerEntryType": "Vault",
            "Flags": 0,
            "Owner": "rVaultOwner123",
            "Account": "rPseudoAccount456",
            "Asset": {"currency": "XRP"},
            "WithdrawalPolicy": 1,
            "Sequence": 5,
            "ShareMPTID": "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E",
            "OwnerNode": "0",
            "PreviousTxnID": "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890",
            "PreviousTxnLgrSeq": 12345678,
            "shares": {
                "LedgerEntryType": "MPTokenIssuance",
                "OutstandingAmount": "1000000"
            }
        },
        "ledger_index": 1000,
        "validated": true
    }"#;

    #[test]
    fn test_vault_info_deserialize() {
        let result: VaultInfo = serde_json::from_str(VAULT_JSON).unwrap();

        assert_eq!(result.ledger_index, Some(1000));
        assert_eq!(result.validated, Some(true));

        let vault_obj = result.vault.unwrap();
        assert_eq!(vault_obj.vault.owner.as_ref(), "rVaultOwner123");
        assert_eq!(vault_obj.vault.account.as_ref(), "rPseudoAccount456");
        assert_eq!(vault_obj.vault.withdrawal_policy, 1);
        assert_eq!(vault_obj.vault.sequence, 5);
        assert_eq!(
            vault_obj.vault.share_mpt_id.as_ref(),
            "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E"
        );

        assert!(
            matches!(vault_obj.vault.asset, Currency::XRP(_)),
            "expected XRP asset, got {:?}",
            vault_obj.vault.asset
        );

        let shares = vault_obj.shares.unwrap();
        assert_eq!(shares.ledger_entry_type.as_deref(), Some("MPTokenIssuance"));
        assert_eq!(shares.outstanding_amount.as_deref(), Some("1000000"));
    }

    #[test]
    fn test_vault_info_round_trip() {
        let info: VaultInfo = serde_json::from_str(VAULT_JSON).unwrap();

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: VaultInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info, deserialized);
        assert!(
            serialized.contains("\"vault\""),
            "expected vault key: {serialized}"
        );
        assert!(
            !serialized.contains("\"node\""),
            "node key must be absent: {serialized}"
        );
    }

    #[test]
    fn test_vault_info_no_shares() {
        let json = r#"{
            "vault": {
                "LedgerEntryType": "Vault",
                "Flags": 0,
                "Owner": "rOwner",
                "Account": "rAccount",
                "Asset": {"currency": "XRP"},
                "WithdrawalPolicy": 1,
                "Sequence": 1,
                "ShareMPTID": "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E",
                "OwnerNode": "0",
                "PreviousTxnID": "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890",
                "PreviousTxnLgrSeq": 1
            },
            "ledger_index": 42,
            "validated": true
        }"#;

        let result: VaultInfo = serde_json::from_str(json).unwrap();
        let vault_obj = result.vault.unwrap();
        assert!(vault_obj.shares.is_none(), "shares should be absent");
        assert_eq!(vault_obj.vault.owner.as_ref(), "rOwner");
    }

    #[test]
    fn test_vault_info_optional_fields_skipped() {
        let info: VaultInfo = serde_json::from_str(VAULT_JSON).unwrap();
        let serialized = serde_json::to_string(&info).unwrap();

        // Optional fields not present in source JSON must not appear in output
        assert!(
            !serialized.contains("\"ledger_current_index\""),
            "absent optional field must be omitted: {serialized}"
        );
    }
}
