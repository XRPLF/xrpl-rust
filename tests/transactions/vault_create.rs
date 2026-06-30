// XLS-65 SingleAssetVault — lifecycle integration tests
//
// Tests mirror xrpl.js packages/xrpl/test/integration/transactions/singleAssetVault.test.ts:
//   - test_vault_lifecycle_iou: full IOU vault lifecycle (create/set/deposit/withdraw/clawback/delete)
//   - test_vault_lifecycle_mpt: full MPT vault lifecycle (mirrors the xrpl.js "MPT in vault" case)
//
// Requires an XLS-65-enabled xrpld node (3.2.0+) at localhost:5005.

#[cfg(feature = "integration")]
mod tests {
    use crate::common::vault::{get_vault_id, vault_assets_total, vault_count};
    use crate::common::{
        create_transferable_clawbackable_mptoken_issuance, generate_funded_wallet,
        test_transaction, with_blockchain_lock,
    };
    use xrpl::models::transactions::account_set::{AccountSet, AccountSetFlag};
    use xrpl::models::transactions::mptoken_authorize::MPTokenAuthorize;
    use xrpl::models::transactions::payment::Payment;
    use xrpl::models::transactions::trust_set::TrustSet;
    use xrpl::models::transactions::vault_clawback::VaultClawback;
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::vault_delete::VaultDelete;
    use xrpl::models::transactions::vault_deposit::VaultDeposit;
    use xrpl::models::transactions::vault_set::VaultSet;
    use xrpl::models::transactions::vault_withdraw::VaultWithdraw;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::{
        Amount, Currency, IssuedCurrency, IssuedCurrencyAmount, MPTAmount, MPTCurrency,
    };

    /// Full IOU vault lifecycle:
    /// AccountSet (DefaultRipple + AllowClawback) → TrustSet → Payment →
    /// VaultCreate → VaultSet → VaultDeposit → VaultWithdraw → VaultClawback → VaultDelete
    #[tokio::test]
    async fn test_vault_lifecycle_iou() {
        with_blockchain_lock(|| async {
            let issuer = generate_funded_wallet().await;
            let vault_owner = generate_funded_wallet().await;
            let holder = generate_funded_wallet().await;
            let currency = "USD";

            // 1. Enable DefaultRipple on issuer
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

            // 2. Enable clawback on issuer
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

            // 3. Holder establishes trust line
            let mut trust = TrustSet {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::TrustSet,
                    ..Default::default()
                },
                limit_amount: IssuedCurrencyAmount::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                    "100000000".into(),
                ),
                ..Default::default()
            };
            test_transaction(&mut trust, &holder).await;

            // 4. Issuer sends USD to holder
            let mut payment = Payment {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::Payment,
                    ..Default::default()
                },
                destination: holder.classic_address.clone().into(),
                amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                    "1000".into(),
                )),
                ..Default::default()
            };
            test_transaction(&mut payment, &issuer).await;

            // 5. VaultCreate
            let mut vault_create = VaultCreate {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultCreate,
                    ..Default::default()
                },
                asset: Currency::IssuedCurrency(IssuedCurrency::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                )),
                withdrawal_policy: Some(1),
                data: Some(hex::encode("vault metadata").to_uppercase().into()),
                mptoken_metadata: Some(hex::encode("share metadata").to_uppercase().into()),
                assets_maximum: Some("9999900000000000000000000".into()),
                scale: Some(2),
                ..Default::default()
            };
            test_transaction(&mut vault_create, &vault_owner).await;

            let vault_id = get_vault_id(&vault_owner.classic_address).await;

            // 6. VaultSet — update AssetsMaximum and Data
            let mut vault_set = VaultSet {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultSet,
                    fee: None,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                assets_maximum: Some("1000".into()),
                data: Some(hex::encode("updated metadata").to_uppercase().into()),
                domain_id: None,
            };
            test_transaction(&mut vault_set, &vault_owner).await;

            // 7. VaultDeposit — deposit 10 USD
            let mut vault_deposit = VaultDeposit {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultDeposit,
                    fee: None,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                    "10".into(),
                )),
            };
            test_transaction(&mut vault_deposit, &holder).await;

            let after_deposit = vault_assets_total(&vault_owner.classic_address).await;
            assert_eq!(after_deposit, "10", "AssetsTotal after deposit");

            // 8. VaultWithdraw — withdraw 5 USD
            let mut vault_withdraw = VaultWithdraw {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultWithdraw,
                    fee: None,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                    "5".into(),
                )),
                destination: Some(holder.classic_address.clone().into()),
                destination_tag: Some(10),
            };
            test_transaction(&mut vault_withdraw, &holder).await;

            let after_withdraw = vault_assets_total(&vault_owner.classic_address).await;
            assert_eq!(after_withdraw, "5", "AssetsTotal after withdrawal");

            // 9. VaultClawback — claw back 5 USD
            let mut vault_clawback = VaultClawback {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultClawback,
                    fee: None,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                holder: holder.classic_address.clone().into(),
                amount: Some(Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    currency.into(),
                    issuer.classic_address.clone().into(),
                    "5".into(),
                ))),
            };
            test_transaction(&mut vault_clawback, &issuer).await;

            // 10. VaultDelete
            let mut vault_delete = VaultDelete {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultDelete,
                    fee: None,
                    ..Default::default()
                },
                vault_id: vault_id.into(),
            };
            test_transaction(&mut vault_delete, &vault_owner).await;

            assert_eq!(
                vault_count(&vault_owner.classic_address).await,
                0,
                "vault should be deleted"
            );
        })
        .await;
    }

    /// Full MPT vault lifecycle, mirroring the xrpl.js "MPT in vault" case:
    /// MPTokenIssuanceCreate (transfer+clawback) → MPTokenAuthorize → Payment →
    /// VaultCreate (MPT asset) → VaultSet → VaultDeposit → VaultWithdraw →
    /// VaultClawback → VaultDelete, asserting AssetsTotal at each step.
    #[tokio::test]
    async fn test_vault_lifecycle_mpt() {
        use xrpl::models::transactions::vault_create::VaultCreateFlag;
        use xrpl::models::FlagCollection;

        with_blockchain_lock(|| async {
            let issuer = generate_funded_wallet().await;
            let vault_owner = generate_funded_wallet().await;
            let holder = generate_funded_wallet().await;

            // 1. Issuer creates a transferable + clawbackable MPT issuance.
            let mpt_id = create_transferable_clawbackable_mptoken_issuance(&issuer).await;

            // 2. Holder opts in to hold the MPT.
            let mut auth = MPTokenAuthorize {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenAuthorize,
                    ..Default::default()
                },
                mptoken_issuance_id: mpt_id.clone().into(),
                holder: None,
            };
            test_transaction(&mut auth, &holder).await;

            // 3. Issuer sends 1000 MPT to the holder.
            let mut payment = Payment {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::Payment,
                    ..Default::default()
                },
                destination: holder.classic_address.clone().into(),
                amount: Amount::MPTAmount(MPTAmount {
                    value: "1000".into(),
                    mpt_issuance_id: mpt_id.clone().into(),
                }),
                ..Default::default()
            };
            test_transaction(&mut payment, &issuer).await;

            // 4. VaultCreate with an MPT asset (no Scale — rejected for MPT vaults).
            let mut vault_create = VaultCreate {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultCreate,
                    flags: FlagCollection::from(vec![VaultCreateFlag::TfVaultShareNonTransferable]),
                    ..Default::default()
                },
                asset: Currency::MPTCurrency(MPTCurrency::new(mpt_id.clone().into())),
                withdrawal_policy: Some(1),
                data: Some(hex::encode("vault metadata").to_uppercase().into()),
                mptoken_metadata: Some(hex::encode("share metadata").to_uppercase().into()),
                assets_maximum: Some("500".into()),
                ..Default::default()
            };
            test_transaction(&mut vault_create, &vault_owner).await;

            let vault_id = get_vault_id(&vault_owner.classic_address).await;

            // 5. VaultSet — raise AssetsMaximum and update Data.
            let mut vault_set = VaultSet {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultSet,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                assets_maximum: Some("1000".into()),
                data: Some(hex::encode("updated metadata").to_uppercase().into()),
                domain_id: None,
            };
            test_transaction(&mut vault_set, &vault_owner).await;

            // 6. VaultDeposit — deposit 10 MPT.
            let mut vault_deposit = VaultDeposit {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultDeposit,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                amount: Amount::MPTAmount(MPTAmount {
                    value: "10".into(),
                    mpt_issuance_id: mpt_id.clone().into(),
                }),
            };
            test_transaction(&mut vault_deposit, &holder).await;
            assert_eq!(
                vault_assets_total(&vault_owner.classic_address).await,
                "10",
                "AssetsTotal after MPT deposit"
            );

            // 7. VaultWithdraw — withdraw 5 MPT.
            let mut vault_withdraw = VaultWithdraw {
                common_fields: CommonFields {
                    account: holder.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultWithdraw,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                amount: Amount::MPTAmount(MPTAmount {
                    value: "5".into(),
                    mpt_issuance_id: mpt_id.clone().into(),
                }),
                destination: Some(holder.classic_address.clone().into()),
                destination_tag: None,
            };
            test_transaction(&mut vault_withdraw, &holder).await;
            assert_eq!(
                vault_assets_total(&vault_owner.classic_address).await,
                "5",
                "AssetsTotal after MPT withdrawal"
            );

            // 8. VaultClawback — claw back the remaining 5 MPT.
            let mut vault_clawback = VaultClawback {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultClawback,
                    ..Default::default()
                },
                vault_id: vault_id.clone().into(),
                holder: holder.classic_address.clone().into(),
                amount: Some(Amount::MPTAmount(MPTAmount {
                    value: "5".into(),
                    mpt_issuance_id: mpt_id.clone().into(),
                })),
            };
            test_transaction(&mut vault_clawback, &issuer).await;
            assert_eq!(
                vault_assets_total(&vault_owner.classic_address).await,
                "0",
                "AssetsTotal after MPT clawback"
            );

            // 9. VaultDelete.
            let mut vault_delete = VaultDelete {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultDelete,
                    ..Default::default()
                },
                vault_id: vault_id.into(),
            };
            test_transaction(&mut vault_delete, &vault_owner).await;
            assert_eq!(
                vault_count(&vault_owner.classic_address).await,
                0,
                "MPT vault should be deleted"
            );
        })
        .await;
    }
}
