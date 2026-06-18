use crate::models::ledger::objects::LedgerEntryType;
use crate::models::{Currency, NoFlags};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `Vault` object type describes a single-asset vault instance (XLS-65).
///
/// All string fields use `Cow<'a, str>`. Vault objects are constructed by the
/// server; callers should treat all fields as read-only.
///
/// Note: the ideal field type for server-read-only strings is `&'a str`
/// (zero-copy, immutable). However, switching to `&'a str` with
/// `#[serde(borrow)]` requires the `'de: 'a` lifetime constraint to propagate
/// through the entire `LedgerEntry` → `BaseLedger` → `LedgerV1` chain, which
/// affects all other ledger objects. A follow-up PR should migrate `Vault`,
/// `AccountRoot`, `MPToken`, and `MPTokenIssuance` to `&'a str` together as
/// part of a codebase-wide ledger-object cleanup.
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065-single-asset-vault>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Vault<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The account address of the Vault Owner. (SoeRequired)
    pub owner: Cow<'a, str>,
    /// The address of the Vault's pseudo-account. (SoeRequired)
    pub account: Cow<'a, str>,
    /// The asset of the vault (XRP, IOU or MPT). (SoeRequired)
    pub asset: Currency<'a>,
    /// The total value of the vault. (SoeDefault)
    pub assets_total: Option<Cow<'a, str>>,
    /// The asset amount that is available in the vault. (SoeDefault)
    pub assets_available: Option<Cow<'a, str>>,
    /// The maximum asset amount that can be held in the vault. Zero means no cap. (SoeOptional)
    pub assets_maximum: Option<Cow<'a, str>>,
    /// The potential loss amount that is not yet realized, expressed as the vault's asset. (SoeDefault)
    pub loss_unrealized: Option<Cow<'a, str>>,
    /// The identifier of the share MPTokenIssuance object. (SoeRequired)
    #[serde(rename = "ShareMPTID")]
    pub share_mpt_id: Cow<'a, str>,
    /// Indicates the withdrawal strategy used by the Vault. (SoeRequired)
    pub withdrawal_policy: u8,
    /// The Scale specifies the power of 10 to multiply an asset's value by
    /// when converting it into an integer-based number of shares. (SoeDefault)
    pub scale: Option<u8>,
    /// The transaction sequence number that created the vault. (SoeRequired)
    pub sequence: u32,
    /// Arbitrary metadata about the Vault. Limited to 256 bytes. (SoeOptional)
    pub data: Option<Cow<'a, str>>,
    /// A hint indicating which page of the owner's directory links to this object. (SoeRequired)
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified
    /// this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> LedgerObject<NoFlags> for Vault<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::currency::{Currency, IssuedCurrency};
    use crate::models::ledger::objects::vault::Vault;
    use crate::models::ledger::objects::CommonFields;
    use crate::models::ledger::objects::LedgerEntryType;
    use crate::models::{FlagCollection, NoFlags};
    use alloc::borrow::Cow;

    fn make_vault<'a>(
        index: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        account: Cow<'a, str>,
        asset: Currency<'a>,
        share_mpt_id: Cow<'a, str>,
        withdrawal_policy: u8,
        sequence: u32,
        owner_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Vault<'a> {
        Vault {
            common_fields: CommonFields {
                flags: FlagCollection::<NoFlags>::default(),
                ledger_entry_type: LedgerEntryType::Vault,
                index,
                ledger_index: None,
            },
            owner,
            account,
            asset,
            assets_total: None,
            assets_available: None,
            assets_maximum: None,
            loss_unrealized: None,
            share_mpt_id,
            withdrawal_policy,
            scale: None,
            sequence,
            data: None,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }

    #[test]
    fn test_serialize() {
        let vault = Vault {
            common_fields: CommonFields {
                flags: FlagCollection::<NoFlags>::default(),
                ledger_entry_type: LedgerEntryType::Vault,
                index: Some(Cow::from("ForTest")),
                ledger_index: None,
            },
            owner: "rVaultOwner123".into(),
            account: "rPseudoAccount456".into(),
            asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            assets_total: Some("1000000".into()),
            assets_available: Some("800000".into()),
            assets_maximum: Some("5000000".into()),
            loss_unrealized: Some("0".into()),
            share_mpt_id: "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E".into(),
            withdrawal_policy: 1,
            scale: Some(6),
            sequence: 5,
            data: Some("48656C6C6F".into()),
            owner_node: "0".into(),
            previous_txn_id: "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"
                .into(),
            previous_txn_lgr_seq: 12345678,
        };

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_minimal_vault() {
        let vault = make_vault(
            Some(Cow::from("MinimalTest")),
            "rMinimalOwner789".into(),
            "rMinimalPseudo789".into(),
            Currency::IssuedCurrency(IssuedCurrency::new("EUR".into(), "rEURIssuer012".into())),
            "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E".into(),
            1,
            1,
            "0".into(),
            "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF".into(),
            1,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_vault_with_all_fields() {
        let vault = Vault {
            common_fields: CommonFields {
                flags: FlagCollection::<NoFlags>::default(),
                ledger_entry_type: LedgerEntryType::Vault,
                index: Some(Cow::from("FullVaultTest")),
                ledger_index: Some(Cow::from("ledger_idx_123")),
            },
            owner: "rFullVaultOwner456".into(),
            account: "rFullPseudoAccount".into(),
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "BTC".into(),
                "rBTCIssuer789".into(),
            )),
            assets_total: Some("50000000".into()),
            assets_available: Some("45000000".into()),
            assets_maximum: Some("100000000".into()),
            loss_unrealized: Some("200000".into()),
            share_mpt_id: "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E".into(),
            withdrawal_policy: 1,
            scale: Some(6),
            sequence: 1,
            data: Some("44617461".into()),
            owner_node: "42".into(),
            previous_txn_id: "FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321"
                .into(),
            previous_txn_lgr_seq: 99999999,
        };

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_serialized_keys_are_pascal_case() {
        let vault = Vault {
            common_fields: CommonFields {
                flags: FlagCollection::<NoFlags>::default(),
                ledger_entry_type: LedgerEntryType::Vault,
                index: Some(Cow::from("KeysTest")),
                ledger_index: None,
            },
            owner: "rKeysOwner".into(),
            account: "rKeysAccount".into(),
            asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuerX".into())),
            assets_total: Some("100".into()),
            assets_available: Some("90".into()),
            assets_maximum: Some("200".into()),
            loss_unrealized: Some("5".into()),
            share_mpt_id: "00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E".into(),
            withdrawal_policy: 1,
            scale: Some(6),
            sequence: 1,
            data: Some("48656C6C6F".into()),
            owner_node: "0".into(),
            previous_txn_id: "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"
                .into(),
            previous_txn_lgr_seq: 100,
        };

        let json = serde_json::to_string(&vault).unwrap();
        assert!(json.contains("\"Account\""), "missing Account key: {json}");
        assert!(json.contains("\"Owner\""), "missing Owner key: {json}");
        assert!(json.contains("\"Asset\""), "missing Asset key: {json}");
        assert!(
            json.contains("\"AssetsTotal\""),
            "missing AssetsTotal key: {json}"
        );
        assert!(
            json.contains("\"AssetsAvailable\""),
            "missing AssetsAvailable key: {json}"
        );
        assert!(
            json.contains("\"AssetsMaximum\""),
            "missing AssetsMaximum key: {json}"
        );
        assert!(
            json.contains("\"LossUnrealized\""),
            "missing LossUnrealized key: {json}"
        );
        assert!(
            json.contains("\"ShareMPTID\""),
            "missing ShareMPTID key: {json}"
        );
        assert!(
            json.contains("\"WithdrawalPolicy\""),
            "missing WithdrawalPolicy key: {json}"
        );
        assert!(json.contains("\"Scale\""), "missing Scale key: {json}");
        assert!(
            json.contains("\"Sequence\""),
            "missing Sequence key: {json}"
        );
        assert!(json.contains("\"Data\""), "missing Data key: {json}");
        assert!(
            json.contains("\"OwnerNode\""),
            "missing OwnerNode key: {json}"
        );
        assert!(
            json.contains("\"PreviousTxnID\""),
            "missing PreviousTxnID key: {json}"
        );
        assert!(
            json.contains("\"PreviousTxnLgrSeq\""),
            "missing PreviousTxnLgrSeq key: {json}"
        );
        assert!(
            json.contains("\"LedgerEntryType\":\"Vault\""),
            "missing LedgerEntryType=Vault: {json}"
        );
    }
}
