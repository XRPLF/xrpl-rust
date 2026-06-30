// XLS-65 SingleAssetVault — VaultSet integration tests
//
// Mirrors xrpl.js singleAssetVault.test.ts VaultSet assertions:
//   - AssetsMaximum updated on ledger after VaultSet
//   - Data (arbitrary metadata) updated on ledger after VaultSet
//
// Requires an XLS-65-enabled xrpld node (3.2.0+) at localhost:5005.

#[cfg(feature = "integration")]
mod tests {
    use crate::common::vault::{account_objects_json, get_vault_id};
    use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::vault_set::VaultSet;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::Currency;

    async fn create_xrp_vault_with_max(
        owner: &xrpl::wallet::Wallet,
        assets_maximum: &str,
    ) -> String {
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::default(),
            withdrawal_policy: Some(1),
            assets_maximum: Some(assets_maximum.into()),
            data: Some(hex::encode("initial metadata").to_uppercase().into()),
            mptoken_metadata: Some(hex::encode("share metadata").to_uppercase().into()),
            ..Default::default()
        };
        test_transaction(&mut vault_create, owner).await;
        get_vault_id(owner.classic_address.as_str()).await
    }

    /// VaultSet updates `AssetsMaximum` on the ledger.
    ///
    /// Mirrors xrpl.js singleAssetVault.test.ts VaultSet step assertion.
    #[tokio::test]
    async fn test_vault_set_updates_assets_maximum() {
        with_blockchain_lock(|| async {
            let owner = generate_funded_wallet().await;
            let vault_id = create_xrp_vault_with_max(&owner, "1000000000").await;

            let mut vault_set = VaultSet {
                common_fields: CommonFields {
                    account: owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultSet,
                    ..Default::default()
                },
                vault_id: vault_id.as_str().into(),
                assets_maximum: Some("5000".into()),
                ..Default::default()
            };
            test_transaction(&mut vault_set, &owner).await;

            let resp = account_objects_json(owner.classic_address.as_str()).await;
            let objects = resp["account_objects"]
                .as_array()
                .expect("account_objects array missing");
            assert!(
                !objects.is_empty(),
                "no vault found for owner after VaultSet"
            );
            let vault = &objects[0];
            assert_eq!(
                vault["AssetsMaximum"].as_str(),
                Some("5000"),
                "AssetsMaximum should be updated to 5000 after VaultSet"
            );
        })
        .await;
    }

    /// VaultSet updates the `Data` field on the ledger.
    ///
    /// Mirrors xrpl.js singleAssetVault.test.ts VaultSet data assertion.
    #[tokio::test]
    async fn test_vault_set_updates_data() {
        with_blockchain_lock(|| async {
            let owner = generate_funded_wallet().await;
            let vault_id = create_xrp_vault_with_max(&owner, "1000000000").await;

            let new_data = hex::encode("updated metadata").to_uppercase();
            let mut vault_set = VaultSet {
                common_fields: CommonFields {
                    account: owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultSet,
                    ..Default::default()
                },
                vault_id: vault_id.as_str().into(),
                data: Some(new_data.as_str().into()),
                ..Default::default()
            };
            test_transaction(&mut vault_set, &owner).await;

            let resp = account_objects_json(owner.classic_address.as_str()).await;
            let objects = resp["account_objects"]
                .as_array()
                .expect("account_objects array missing");
            assert!(
                !objects.is_empty(),
                "no vault found for owner after VaultSet"
            );
            let vault = &objects[0];
            assert_eq!(
                vault["Data"].as_str(),
                Some(new_data.as_str()),
                "Data field should reflect updated value after VaultSet"
            );
        })
        .await;
    }
}
