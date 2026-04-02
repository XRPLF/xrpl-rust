use crate::models::ledger::objects::LedgerEntryType;
use crate::models::{Currency, FlagCollection, Model, NoFlags};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `Vault` object type describes a single-asset vault instance (XLS-65).
///
/// A vault holds a single asset type and issues share tokens (MPTokens)
/// to depositors proportional to their ownership of the vault's assets.
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
    /// The account address of the Vault Owner.
    pub owner: Cow<'a, str>,
    /// The address of the Vault's pseudo-account.
    pub account: Cow<'a, str>,
    /// The asset of the vault (XRP, IOU or MPT).
    pub asset: Currency<'a>,
    /// The total value of the vault.
    pub assets_total: Option<Cow<'a, str>>,
    /// The asset amount that is available in the vault.
    pub assets_available: Option<Cow<'a, str>>,
    /// The maximum asset amount that can be held in the vault. Zero means no cap.
    pub assets_maximum: Option<Cow<'a, str>>,
    /// The potential loss amount that is not yet realized, expressed as the vault's asset.
    pub loss_unrealized: Option<Cow<'a, str>>,
    /// The identifier of the share MPTokenIssuance object.
    #[serde(rename = "ShareMPTID")]
    pub share_mpt_id: Option<Cow<'a, str>>,
    /// Indicates the withdrawal strategy used by the Vault.
    pub withdrawal_policy: Option<u8>,
    /// The Scale specifies the power of 10 to multiply an asset's value by
    /// when converting it into an integer-based number of shares.
    pub scale: Option<u8>,
    /// The transaction sequence number that created the vault.
    pub sequence: Option<u32>,
    /// Arbitrary metadata about the Vault. Limited to 256 bytes.
    pub data: Option<Cow<'a, str>>,
    /// A hint indicating which page of the owner's directory links to this object.
    pub owner_node: Option<Cow<'a, str>>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently modified
    /// this object.
    pub previous_txn_lgr_seq: u32,
}

impl<'a> Model for Vault<'a> {}

impl<'a> LedgerObject<NoFlags> for Vault<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Vault<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        account: Cow<'a, str>,
        asset: Currency<'a>,
        assets_total: Option<Cow<'a, str>>,
        assets_available: Option<Cow<'a, str>>,
        assets_maximum: Option<Cow<'a, str>>,
        loss_unrealized: Option<Cow<'a, str>>,
        share_mpt_id: Option<Cow<'a, str>>,
        withdrawal_policy: Option<u8>,
        scale: Option<u8>,
        sequence: Option<u32>,
        data: Option<Cow<'a, str>>,
        owner_node: Option<Cow<'a, str>>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Vault,
                index,
                ledger_index,
            },
            owner,
            account,
            asset,
            assets_total,
            assets_available,
            assets_maximum,
            loss_unrealized,
            share_mpt_id,
            withdrawal_policy,
            scale,
            sequence,
            data,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::currency::{Currency, IssuedCurrency, XRP};
    use crate::models::ledger::objects::vault::Vault;
    use alloc::borrow::Cow;

    #[test]
    fn test_serialize() {
        let vault = Vault::new(
            Some(Cow::from("ForTest")),
            None,
            Cow::from("rVaultOwner123"),
            Cow::from("rPseudoAccount456"),
            Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            Some("1000000".into()),
            Some("800000".into()),
            Some("5000000".into()),
            Some("0".into()),
            Some("00000001C752C42A1EBD6BF2403134F7CFD2F1D835AFD26E".into()),
            Some(1),
            Some(6),
            Some(5),
            Some("48656C6C6F".into()),
            Some("0".into()),
            Cow::from("ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"),
            12345678,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_minimal_vault() {
        let vault = Vault::new(
            Some(Cow::from("MinimalTest")),
            None,
            Cow::from("rMinimalOwner789"),
            Cow::from("rMinimalPseudo789"),
            Currency::IssuedCurrency(IssuedCurrency::new("EUR".into(), "rEURIssuer012".into())),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Cow::from("1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF"),
            1,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_vault_with_all_fields() {
        let vault = Vault::new(
            Some(Cow::from("FullVaultTest")),
            Some(Cow::from("ledger_idx_123")),
            Cow::from("rFullVaultOwner456"),
            Cow::from("rFullPseudoAccount"),
            Currency::IssuedCurrency(IssuedCurrency::new("BTC".into(), "rBTCIssuer789".into())),
            Some("50000000".into()),
            Some("45000000".into()),
            Some("100000000".into()),
            Some("200000".into()),
            Some("0000000000000001".into()),
            Some(1),
            Some(6),
            Some(1),
            Some("44617461".into()),
            Some("42".into()),
            Cow::from("FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321"),
            99999999,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }

    #[test]
    fn test_xrp_vault() {
        let vault = Vault::new(
            Some(Cow::from("XRPVaultTest")),
            None,
            Cow::from("rwhaYGnJMexktjhxAKzRwoCcQ2g6hvBDWu"),
            Cow::from("rBVxExjRR6oDMWCeQYgJP7q4JBLGeLBPyv"),
            Currency::XRP(XRP::new()),
            Some("0".into()),
            Some("0".into()),
            None,
            Some("0".into()),
            Some("00000001732B0822A31109C996BCDD7E64E05D446E7998EE".into()),
            Some(1),
            Some(0),
            Some(4),
            None,
            Some("0".into()),
            Cow::from("25C3C8BF2C9EE60DFCDA02F3919D0C4D6BF2D0A4AC9354EFDA438F2ECDDA65E4"),
            5,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }
}
