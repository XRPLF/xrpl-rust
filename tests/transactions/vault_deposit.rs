// XLS-65 SingleAssetVault — VaultDeposit integration tests
//
// Mirrors rippled Vault_test.cpp tfVaultPrivate deposit restriction:
//   - Non-allowlisted account cannot deposit into a private vault
//
// Requires an XLS-65-enabled xrpld node (3.2.0+) at localhost:5005.

#[cfg(feature = "integration")]
mod tests {
    use crate::common::vault::get_vault_id;
    use crate::common::{
        generate_funded_wallet, submit_tx, test_transaction, with_blockchain_lock, SubmitOptions,
    };
    use xrpl::models::transactions::vault_create::{VaultCreate, VaultCreateFlag};
    use xrpl::models::transactions::vault_deposit::VaultDeposit;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::{Amount, Currency, FlagCollection, XRPAmount};

    /// A non-allowlisted account cannot deposit into a private vault.
    ///
    /// Mirrors rippled Vault_test.cpp tfVaultPrivate restriction assertions
    /// and xrpl.js singleAssetVault.test.ts private-vault block.
    #[tokio::test]
    async fn test_vault_deposit_private_vault_rejects_unauthorized() {
        with_blockchain_lock(|| async {
            let owner = generate_funded_wallet().await;
            let outsider = generate_funded_wallet().await;

            // Create a private XRP vault: only allowlisted accounts may deposit.
            let mut vault_create = VaultCreate {
                common_fields: CommonFields {
                    account: owner.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultCreate,
                    flags: FlagCollection::from(vec![VaultCreateFlag::TfVaultPrivate]),
                    ..Default::default()
                },
                asset: Currency::default(),
                withdrawal_policy: Some(1),
                ..Default::default()
            };
            test_transaction(&mut vault_create, &owner).await;

            let vault_id = get_vault_id(owner.classic_address.as_str()).await;

            // Outsider (not on the vault's domain allow-list) attempts to deposit.
            let mut deposit = VaultDeposit {
                common_fields: CommonFields {
                    account: outsider.classic_address.clone().into(),
                    transaction_type: TransactionType::VaultDeposit,
                    ..Default::default()
                },
                vault_id: vault_id.as_str().into(),
                amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            };

            let result = submit_tx(
                &mut deposit,
                SubmitOptions {
                    wallet: &outsider,
                    autofill: true,
                    check_fee: true,
                },
            )
            .await;

            assert_eq!(
                result, "tecNO_AUTH",
                "deposit into private vault by non-allowlisted account must return tecNO_AUTH, got: {result}"
            );
        })
        .await;
    }
}
