// XLS-65 SingleAssetVault — VaultClawback integration tests
//
// Tests mirror:
//   xrpl.js singleAssetVault.test.ts (partial + full clawback with AssetsTotal assertion)
//   rippled Vault_test.cpp "clawback all", "only issuer can clawback"
//
// Requires an XLS-65-enabled xrpld node (3.2.0+) at localhost:5005.

#[cfg(feature = "integration")]
mod tests {
    use crate::common::{
        generate_funded_wallet, get_client, submit_tx, test_transaction, with_blockchain_lock,
        SubmitOptions,
    };
    use serde_json::Value;
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
    use xrpl::models::requests::{CommonFields as ReqCommonFields, RequestMethod};
    use xrpl::models::transactions::account_set::{AccountSet, AccountSetFlag};
    use xrpl::models::transactions::payment::Payment;
    use xrpl::models::transactions::trust_set::TrustSet;
    use xrpl::models::transactions::vault_clawback::VaultClawback;
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::vault_deposit::VaultDeposit;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::{Amount, Currency, IssuedCurrency, IssuedCurrencyAmount};
    use xrpl::wallet::Wallet;

    // ── helpers ──────────────────────────────────────────────────────────────

    async fn vault_ao_json(owner: &str) -> Value {
        let client = get_client().await;
        let resp = client
            .request(
                AccountObjects {
                    common_fields: ReqCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: owner.into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::Vault),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request failed");
        resp.raw_result.unwrap_or(Value::Null)
    }

    async fn get_vault_id(owner: &str) -> String {
        let raw = vault_ao_json(owner).await;
        raw["account_objects"][0]["index"]
            .as_str()
            .expect("vault index missing")
            .to_string()
    }

    async fn vault_assets_total(owner: &str) -> String {
        let raw = vault_ao_json(owner).await;
        raw["account_objects"][0]["AssetsTotal"]
            .as_str()
            .unwrap_or("0")
            .to_string()
    }

    /// Create issuer + vault_owner + holder, fund trust line, deposit `deposit_amount` USD.
    /// Returns (issuer, vault_owner, holder, vault_id).
    async fn setup_iou_vault_with_deposit(
        deposit_amount: &str,
    ) -> (Wallet, Wallet, Wallet, String) {
        const CURRENCY: &str = "USD";

        let issuer = generate_funded_wallet().await;
        let vault_owner = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;

        let mut tx = AccountSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            set_flag: Some(AccountSetFlag::AsfDefaultRipple),
            ..Default::default()
        };
        test_transaction(&mut tx, &issuer).await;

        let mut tx = AccountSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            set_flag: Some(AccountSetFlag::AsfAllowTrustLineClawback),
            ..Default::default()
        };
        test_transaction(&mut tx, &issuer).await;

        let mut trust = TrustSet {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                CURRENCY.into(),
                issuer.classic_address.clone().into(),
                "100000000".into(),
            ),
            ..Default::default()
        };
        test_transaction(&mut trust, &holder).await;

        let mut payment = Payment {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            destination: holder.classic_address.clone().into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                CURRENCY.into(),
                issuer.classic_address.clone().into(),
                "1000".into(),
            )),
            ..Default::default()
        };
        test_transaction(&mut payment, &issuer).await;

        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                CURRENCY.into(),
                issuer.classic_address.clone().into(),
            )),
            withdrawal_policy: Some(1),
            assets_maximum: Some("999999999".into()),
            scale: Some(2),
            ..Default::default()
        };
        test_transaction(&mut vault_create, &vault_owner).await;

        let vault_id = get_vault_id(&vault_owner.classic_address).await;

        let mut deposit = VaultDeposit {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::VaultDeposit,
                ..Default::default()
            },
            vault_id: vault_id.clone().into(),
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                CURRENCY.into(),
                issuer.classic_address.clone().into(),
                deposit_amount.into(),
            )),
        };
        test_transaction(&mut deposit, &holder).await;

        (issuer, vault_owner, holder, vault_id)
    }

    // ── tests ─────────────────────────────────────────────────────────────────

    /// Partial clawback: deposit 10, clawback 5 → AssetsTotal == 5.
    /// Mirrors xrpl.js singleAssetVault.test.ts IOU lifecycle clawback assertion.
    #[tokio::test]
    async fn test_vault_clawback_partial() {
        with_blockchain_lock(|| async {
            let (issuer, vault_owner, holder, vault_id) = setup_iou_vault_with_deposit("10").await;

            let mut clawback = VaultClawback {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultClawback,
                    ..Default::default()
                },
                vault_id: vault_id.into(),
                holder: holder.classic_address.clone().into(),
                amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    "USD".into(),
                    issuer.classic_address.clone().into(),
                    "5".into(),
                ))),
            };
            test_transaction(&mut clawback, &issuer).await;

            assert_eq!(
                vault_assets_total(&vault_owner.classic_address).await,
                "5",
                "AssetsTotal should be 5 after partial clawback"
            );
        })
        .await;
    }

    /// Full clawback (amount: None): deposit 10, clawback all → AssetsTotal == 0.
    /// Mirrors rippled Vault_test.cpp "clawback all".
    #[tokio::test]
    async fn test_vault_clawback_all() {
        with_blockchain_lock(|| async {
            let (issuer, vault_owner, holder, vault_id) = setup_iou_vault_with_deposit("10").await;

            let mut clawback = VaultClawback {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultClawback,
                    ..Default::default()
                },
                vault_id: vault_id.into(),
                holder: holder.classic_address.clone().into(),
                amount: None,
            };
            test_transaction(&mut clawback, &issuer).await;

            assert_eq!(
                vault_assets_total(&vault_owner.classic_address).await,
                "0",
                "AssetsTotal should be 0 after full clawback"
            );
        })
        .await;
    }

    /// Non-issuer cannot clawback vault assets.
    /// Mirrors rippled Vault_test.cpp "only issuer can clawback".
    #[tokio::test]
    async fn test_vault_clawback_non_issuer_rejected() {
        with_blockchain_lock(|| async {
            let (issuer, _vault_owner, holder, vault_id) = setup_iou_vault_with_deposit("10").await;

            let non_issuer = generate_funded_wallet().await;

            let mut clawback = VaultClawback {
                common_fields: CommonFields {
                    account: non_issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultClawback,
                    ..Default::default()
                },
                vault_id: vault_id.into(),
                holder: holder.classic_address.clone().into(),
                amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    "USD".into(),
                    issuer.classic_address.clone().into(),
                    "5".into(),
                ))),
            };
            let result = submit_tx(
                &mut clawback,
                SubmitOptions {
                    wallet: &non_issuer,
                    autofill: true,
                    check_fee: true,
                },
            )
            .await;
            assert_ne!(
                result, "tesSUCCESS",
                "non-issuer clawback should be rejected, got: {result}"
            );
        })
        .await;
    }
}
