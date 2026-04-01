use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    Currency, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Create a new single-asset vault on the XRP Ledger (XLS-65).
///
/// A vault holds a single asset type and issues share tokens (MPTokens)
/// to depositors proportional to their ownership of the vault's assets.
///
/// See VaultCreate transaction:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
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
pub struct VaultCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The asset that this vault will hold.
    pub asset: Currency<'a>,
    /// Arbitrary hex-encoded data associated with the vault.
    pub data: Option<Cow<'a, str>>,
    /// The maximum amount of assets the vault can hold, as a string-encoded integer.
    pub assets_maximum: Option<Cow<'a, str>>,
    /// Metadata for the MPToken issued by the vault.
    #[serde(rename = "MPTokenMetadata")]
    pub mptoken_metadata: Option<Cow<'a, str>>,
    /// The domain ID associated with the vault.
    #[serde(rename = "DomainID")]
    pub domain_id: Option<Cow<'a, str>>,
    /// The withdrawal policy for the vault.
    /// 0 = unrestricted, 1 = stranded (withdrawals require approval).
    pub withdrawal_policy: Option<u8>,
}

impl Model for VaultCreate<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for VaultCreate<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for VaultCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> VaultCreate<'a> {
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
        asset: Currency<'a>,
        data: Option<Cow<'a, str>>,
        assets_maximum: Option<Cow<'a, str>>,
        mptoken_metadata: Option<Cow<'a, str>>,
        domain_id: Option<Cow<'a, str>>,
        withdrawal_policy: Option<u8>,
    ) -> VaultCreate<'a> {
        VaultCreate {
            common_fields: CommonFields::new(
                account,
                TransactionType::VaultCreate,
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
            asset,
            data,
            assets_maximum,
            mptoken_metadata,
            domain_id,
            withdrawal_policy,
        }
    }

    /// Set the data field.
    pub fn with_data(mut self, data: Cow<'a, str>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the assets maximum field.
    pub fn with_assets_maximum(mut self, assets_maximum: Cow<'a, str>) -> Self {
        self.assets_maximum = Some(assets_maximum);
        self
    }

    /// Set the MPToken metadata field.
    pub fn with_mptoken_metadata(mut self, mptoken_metadata: Cow<'a, str>) -> Self {
        self.mptoken_metadata = Some(mptoken_metadata);
        self
    }

    /// Set the domain ID field.
    pub fn with_domain_id(mut self, domain_id: Cow<'a, str>) -> Self {
        self.domain_id = Some(domain_id);
        self
    }

    /// Set the withdrawal policy field.
    pub fn with_withdrawal_policy(mut self, withdrawal_policy: u8) -> Self {
        self.withdrawal_policy = Some(withdrawal_policy);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::currency::{IssuedCurrency, XRP};

    #[test]
    fn test_serde() {
        let vault_create = VaultCreate {
            common_fields: CommonFields {
                account: "rVaultCreator123".into(),
                transaction_type: TransactionType::VaultCreate,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            data: None,
            assets_maximum: None,
            mptoken_metadata: None,
            domain_id: None,
            withdrawal_policy: None,
        };

        let json_str = r#"{"Account":"rVaultCreator123","TransactionType":"VaultCreate","Flags":0,"SigningPubKey":"","Asset":{"currency":"USD","issuer":"rIssuer456"}}"#;

        // Serialize
        let serialized = serde_json::to_string(&vault_create).unwrap();
        assert_eq!(
            serde_json::to_value(&serialized).unwrap(),
            serde_json::to_value(json_str).unwrap()
        );

        // Deserialize
        let deserialized: VaultCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(vault_create, deserialized);
    }

    #[test]
    fn test_serde_with_all_fields() {
        let vault_create = VaultCreate {
            common_fields: CommonFields {
                account: "rVaultCreator789".into(),
                transaction_type: TransactionType::VaultCreate,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            data: Some("48656C6C6F".into()),
            assets_maximum: Some("1000000000".into()),
            mptoken_metadata: Some("ABCDEF".into()),
            domain_id: Some(
                "D0000000000000000000000000000000000000000000000000000000DEADBEEF".into(),
            ),
            withdrawal_policy: Some(1),
        };

        let serialized = serde_json::to_string(&vault_create).unwrap();
        let deserialized: VaultCreate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault_create, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let vault_create = VaultCreate {
            common_fields: CommonFields {
                account: "rVaultCreator123".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_data("48656C6C6F".into())
        .with_assets_maximum("1000000000".into())
        .with_mptoken_metadata("ABCDEF".into())
        .with_domain_id("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into())
        .with_withdrawal_policy(1)
        .with_memo(Memo {
            memo_data: Some("creating vault".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(vault_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_create.common_fields.sequence, Some(100));
        assert_eq!(
            vault_create.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(vault_create.common_fields.source_tag, Some(12345));
        assert_eq!(vault_create.data, Some("48656C6C6F".into()));
        assert_eq!(vault_create.assets_maximum, Some("1000000000".into()));
        assert_eq!(vault_create.mptoken_metadata, Some("ABCDEF".into()));
        assert_eq!(vault_create.withdrawal_policy, Some(1));
        assert_eq!(vault_create.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let vault_create = VaultCreate {
            common_fields: CommonFields {
                account: "rVaultCreator123".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer456".into())),
            ..Default::default()
        };

        assert_eq!(vault_create.common_fields.account, "rVaultCreator123");
        assert_eq!(
            vault_create.common_fields.transaction_type,
            TransactionType::VaultCreate
        );
        assert!(vault_create.data.is_none());
        assert!(vault_create.assets_maximum.is_none());
        assert!(vault_create.mptoken_metadata.is_none());
        assert!(vault_create.domain_id.is_none());
        assert!(vault_create.withdrawal_policy.is_none());
        assert!(vault_create.common_fields.fee.is_none());
        assert!(vault_create.common_fields.sequence.is_none());
    }

    #[test]
    fn test_xrp_vault() {
        let xrp_vault = VaultCreate {
            common_fields: CommonFields {
                account: "rXRPVaultCreator789".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_assets_maximum("50000000000".into());

        assert!(matches!(xrp_vault.asset, Currency::XRP(_)));
        assert_eq!(xrp_vault.assets_maximum, Some("50000000000".into()));
        assert_eq!(xrp_vault.common_fields.sequence, Some(100));
        assert!(xrp_vault.validate().is_ok());
    }

    #[test]
    fn test_issued_currency_vault() {
        let token_vault = VaultCreate {
            common_fields: CommonFields {
                account: "rTokenVaultCreator111".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rUSDIssuer222".into(),
            )),
            ..Default::default()
        }
        .with_fee("15".into())
        .with_sequence(200)
        .with_withdrawal_policy(0);

        assert!(matches!(token_vault.asset, Currency::IssuedCurrency(_)));
        assert_eq!(token_vault.withdrawal_policy, Some(0));
        assert!(token_vault.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_vault = VaultCreate {
            common_fields: CommonFields {
                account: "rTicketVault333".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            ..Default::default()
        }
        .with_ticket_sequence(12345)
        .with_fee("12".into());

        assert_eq!(ticket_vault.common_fields.ticket_sequence, Some(12345));
        assert!(ticket_vault.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_vault = VaultCreate {
            common_fields: CommonFields {
                account: "rMultiMemoVault444".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "EUR".into(),
                "rEURIssuer555".into(),
            )),
            ..Default::default()
        }
        .with_memo(Memo {
            memo_data: Some("first memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("second memo".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_vault.common_fields.memos.as_ref().unwrap().len(),
            2
        );
        assert_eq!(multi_memo_vault.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_new_constructor() {
        let vault = VaultCreate::new(
            "rNewVaultAccount".into(),
            None,
            Some("12".into()),
            Some(7108682),
            None,
            Some(100),
            None,
            None,
            None,
            Currency::IssuedCurrency(IssuedCurrency::new("USD".into(), "rIssuer789".into())),
            Some("48656C6C6F".into()),
            Some("1000000000".into()),
            Some("ABCDEF".into()),
            Some("D0000000000000000000000000000000000000000000000000000000DEADBEEF".into()),
            Some(1),
        );

        assert_eq!(vault.common_fields.account, "rNewVaultAccount");
        assert_eq!(
            vault.common_fields.transaction_type,
            TransactionType::VaultCreate
        );
        assert_eq!(vault.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(vault.common_fields.sequence, Some(100));
        assert_eq!(vault.data, Some("48656C6C6F".into()));
        assert_eq!(vault.assets_maximum, Some("1000000000".into()));
        assert_eq!(vault.mptoken_metadata, Some("ABCDEF".into()));
        assert_eq!(vault.withdrawal_policy, Some(1));
    }

    #[test]
    fn test_stranded_withdrawal_policy() {
        let stranded_vault = VaultCreate {
            common_fields: CommonFields {
                account: "rStrandedVault666".into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "BTC".into(),
                "rBTCIssuer777".into(),
            )),
            withdrawal_policy: Some(1),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(500);

        assert_eq!(stranded_vault.withdrawal_policy, Some(1));
        assert!(stranded_vault.validate().is_ok());
    }
}
