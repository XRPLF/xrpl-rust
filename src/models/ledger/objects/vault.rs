use crate::models::ledger::objects::LedgerEntryType;
use crate::models::{Amount, Currency, FlagCollection, Model, NoFlags};
use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The `Vault` object type describes a single-asset vault instance (XLS-65).
///
/// A vault holds a single asset type and issues share tokens (MPTokens)
/// to depositors proportional to their ownership of the vault's assets.
///
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
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
    /// The account that created and owns this vault.
    pub account: Cow<'a, str>,
    /// The asset that this vault holds.
    pub asset: Currency<'a>,
    /// The total amount of assets currently held in the vault.
    pub assets_total: Option<Cow<'a, str>>,
    /// The amount of assets available for withdrawal.
    pub assets_available: Option<Cow<'a, str>>,
    /// The maximum amount of assets the vault can hold.
    pub assets_maximum: Option<Cow<'a, str>>,
    /// The liquidity provider token balance for this vault.
    #[serde(rename = "LPToken")]
    pub lp_token: Option<Amount<'a>>,
    /// The share token for this vault.
    pub share: Option<Amount<'a>>,
    /// Arbitrary hex-encoded data associated with the vault.
    pub data: Option<Cow<'a, str>>,
    /// The ID of the MPToken issuance associated with this vault.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mpt_issuance_id: Option<Cow<'a, str>>,
    /// The domain ID associated with the vault.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
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
        account: Cow<'a, str>,
        asset: Currency<'a>,
        assets_total: Option<Cow<'a, str>>,
        assets_available: Option<Cow<'a, str>>,
        assets_maximum: Option<Cow<'a, str>>,
        lp_token: Option<Amount<'a>>,
        share: Option<Amount<'a>>,
        data: Option<Cow<'a, str>>,
        mpt_issuance_id: Option<Cow<'a, str>>,
        domain_id: Option<Cow<'a, str>>,
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
            account,
            asset,
            assets_total,
            assets_available,
            assets_maximum,
            lp_token,
            share,
            data,
            mpt_issuance_id,
            domain_id,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use crate::models::amount::IssuedCurrencyAmount;
    use crate::models::currency::{Currency, IssuedCurrency};
    use crate::models::ledger::objects::vault::Vault;
    use alloc::borrow::Cow;

    #[test]
    fn test_serialize() {
        let vault = Vault::new(
            Some(Cow::from("ForTest")),
            None,
            Cow::from("rVaultOwner123"),
            Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            Some("1000000".into()),
            Some("800000".into()),
            Some("5000000".into()),
            None,
            Some(crate::models::Amount::IssuedCurrencyAmount(
                IssuedCurrencyAmount::new(
                    "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                    "rVaultOwner123".into(),
                    "1000".into(),
                ),
            )),
            Some("48656C6C6F".into()),
            Some("0000000000000001".into()),
            None,
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
            Cow::from("rMinimalVault789"),
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
            Currency::IssuedCurrency(IssuedCurrency::new("BTC".into(), "rBTCIssuer789".into())),
            Some("50000000".into()),
            Some("45000000".into()),
            Some("100000000".into()),
            Some(crate::models::Amount::IssuedCurrencyAmount(
                IssuedCurrencyAmount::new(
                    "LP_TOKEN_CURRENCY".into(),
                    "rFullVaultOwner456".into(),
                    "5000".into(),
                ),
            )),
            Some(crate::models::Amount::IssuedCurrencyAmount(
                IssuedCurrencyAmount::new(
                    "SHARE_CURRENCY".into(),
                    "rFullVaultOwner456".into(),
                    "2500".into(),
                ),
            )),
            Some("44617461".into()),
            Some("00000000DEADBEEF".into()),
            Some("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into()),
            Some("42".into()),
            Cow::from("FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321FEDCBA0987654321"),
            99999999,
        );

        let serialized = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault, deserialized);
    }
}
