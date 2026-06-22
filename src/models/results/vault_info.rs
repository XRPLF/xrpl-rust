use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response from the `vault_info` method (XLS-65 SingleAssetVault).
///
/// xrpld returns the vault object under a `vault` key (not `node`). The
/// nested `shares` sub-object within `vault` carries MPTokenIssuance fields
/// for the share token.  Both are typed as `Option<Value>` until a dedicated
/// `VaultShares` struct is introduced.
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VaultInfo<'a> {
    /// The vault object fields as returned by xrpld (PascalCase keys), with a
    /// nested `shares` object containing the MPTokenIssuance for vault shares.
    pub vault: Option<Value>,
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

    #[test]
    fn test_vault_info_deserialize() {
        let json = r#"{
            "vault": {
                "LedgerEntryType": "Vault",
                "Flags": 0,
                "Owner": "rVaultOwner123",
                "Account": "rPseudoAccount456",
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

        let result: VaultInfo = serde_json::from_str(json).unwrap();
        assert_eq!(result.ledger_index, Some(1000));
        assert_eq!(result.validated, Some(true));
        let vault = result.vault.unwrap();
        assert_eq!(vault["LedgerEntryType"], "Vault");
        assert_eq!(vault["Owner"], "rVaultOwner123");
        assert_eq!(vault["shares"]["LedgerEntryType"], "MPTokenIssuance");
        assert_eq!(vault["shares"]["OutstandingAmount"], "1000000");
    }

    #[test]
    fn test_vault_info_round_trip() {
        let info = VaultInfo {
            vault: Some(serde_json::json!({
                "LedgerEntryType": "Vault",
                "Flags": 65536,
                "Owner": "rOwner",
                "Account": "rAccount",
                "shares": {
                    "LedgerEntryType": "MPTokenIssuance",
                    "OutstandingAmount": "500"
                }
            })),
            ledger_current_index: None,
            ledger_index: Some(42),
            ledger_hash: Some("AABBCC".into()),
            validated: Some(true),
        };

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
}
