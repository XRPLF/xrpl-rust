// xrpl.js reference: packages/xrpl/test/integration/transactions/lendingProtocolV1_1.test.ts
// (XRPLF/xrpl.js#3456)
// Specs: XLS-0058 (https://github.com/XRPLF/XRPL-Standards/pull/582),
//        XLS-0066 V1_1 (https://github.com/XRPLF/XRPL-Standards/pull/587)
//
// Scenarios:
//   - close_ended_vault: VaultCreate with VaultKind=1 + SubscriptionDate /
//     RedemptionDate, verify the fields land on the Vault ledger object
//   - vault_delete_with_memo_data: VaultDelete carrying MemoData, verify the
//     vault is gone afterwards
//   - domain_gated_withdraw: withdraw from a private, domain-gated vault by
//     presenting CredentialIDs (XLS-70)

use crate::common::{
    generate_funded_wallet, get_client, get_ledger_close_time, provision_credential,
    test_transaction,
    vault::{account_objects_json, get_vault_id},
    with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::ledger::objects::vault::Vault;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::transactions::{
    permissioned_domain_set::PermissionedDomainSet,
    vault_create::{VaultCreate, VaultCreateFlag, VaultKind},
    vault_delete::VaultDelete,
    vault_deposit::VaultDeposit,
    vault_withdraw::VaultWithdraw,
    CommonFields, Credential, TransactionType,
};
use xrpl::models::{Amount, Currency, FlagCollection, XRPAmount, XRP};
use xrpl::wallet::Wallet;

/// `SubscriptionDate` / `RedemptionDate` are ledger close times, so they are
/// derived from the latest validated ledger: the standalone node's clock is not
/// in sync with the local system clock.
async fn investment_window() -> (u32, u32) {
    let close_time = get_ledger_close_time().await as u32;
    let subscription_date = close_time + 300;
    (subscription_date, subscription_date + 3600)
}

fn xrp_vault<'a>(owner: &Wallet) -> VaultCreate<'a> {
    VaultCreate {
        common_fields: CommonFields {
            account: owner.classic_address.clone().into(),
            transaction_type: TransactionType::VaultCreate,
            ..Default::default()
        },
        asset: Currency::XRP(XRP::new()),
        withdrawal_policy: Some(1),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_create_close_ended_vault() {
    with_blockchain_lock(|| async {
        let vault_owner = generate_funded_wallet().await;
        let (subscription_date, redemption_date) = investment_window().await;

        let mut tx = xrp_vault(&vault_owner)
            .with_assets_maximum("1000".into())
            .with_vault_kind(VaultKind::Closed)
            .with_subscription_date(subscription_date)
            .with_redemption_date(redemption_date);

        test_transaction(&mut tx, &vault_owner).await;

        let objects = account_objects_json(&vault_owner.classic_address).await;
        let vault = &objects["account_objects"][0];

        assert_eq!(
            vault["Owner"].as_str(),
            Some(vault_owner.classic_address.as_str())
        );
        assert_eq!(vault["VaultKind"].as_u64(), Some(1));
        assert_eq!(
            vault["SubscriptionDate"].as_u64(),
            Some(subscription_date as u64)
        );
        assert_eq!(
            vault["RedemptionDate"].as_u64(),
            Some(redemption_date as u64)
        );

        // The same object has to round-trip through the typed `Vault` ledger
        // model, which is how library users read it.
        let typed: Vault =
            serde_json::from_value(vault.clone()).expect("failed to deserialize Vault");
        assert_eq!(typed.vault_kind, Some(1));
        assert_eq!(typed.subscription_date, Some(subscription_date));
        assert_eq!(typed.redemption_date, Some(redemption_date));
        assert!(
            typed.le_version.is_some(),
            "a LendingProtocolV1_1 server sets LEVersion"
        );
    })
    .await
}

#[tokio::test]
async fn test_delete_vault_with_memo_data() {
    with_blockchain_lock(|| async {
        let vault_owner = generate_funded_wallet().await;

        let mut create = xrp_vault(&vault_owner);
        test_transaction(&mut create, &vault_owner).await;

        let vault_id = get_vault_id(&vault_owner.classic_address).await;

        let mut delete = VaultDelete {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultDelete,
                ..Default::default()
            },
            vault_id: vault_id.clone().into(),
            ..Default::default()
        }
        // hex("closing vault")
        .with_memo_data("636C6F73696E67207661756C74".into());

        test_transaction(&mut delete, &vault_owner).await;

        let objects = account_objects_json(&vault_owner.classic_address).await;
        assert!(
            objects["account_objects"]
                .as_array()
                .expect("account_objects array missing")
                .is_empty(),
            "vault should be deleted from account objects"
        );
    })
    .await
}

#[tokio::test]
async fn test_withdraw_from_domain_gated_vault_with_credential_ids() {
    with_blockchain_lock(|| async {
        let vault_owner = generate_funded_wallet().await;
        let depositor = generate_funded_wallet().await;
        // hex("lp-kyc")
        let credential_type = "6C702D6B7963";

        // The owner issues a credential to the depositor, who accepts it.
        let credential_id = provision_credential(&vault_owner, &depositor, credential_type).await;

        // The owner opens a permissioned domain that accepts that credential.
        let mut pd_set = PermissionedDomainSet {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            accepted_credentials: vec![Credential {
                issuer: vault_owner.classic_address.clone(),
                credential_type: credential_type.to_string(),
            }],
            domain_id: None,
        };
        test_transaction(&mut pd_set, &vault_owner).await;

        let client = get_client().await;
        let pd_resp = client
            .request(
                AccountObjects::new(
                    None,
                    vault_owner.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::PermissionedDomain),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("permissioned_domain account_objects request failed");
        let domain_id = pd_resp
            .raw_result
            .expect("account_objects response contained no raw_result")["account_objects"][0]
            ["index"]
            .as_str()
            .expect("permissioned domain index missing")
            .to_string();

        // A private vault gated on that domain.
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                flags: FlagCollection::new(vec![VaultCreateFlag::TfVaultPrivate]),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            withdrawal_policy: Some(1),
            domain_id: Some(domain_id.clone().into()),
            ..Default::default()
        };
        test_transaction(&mut vault_create, &vault_owner).await;

        let vault_id = get_vault_id(&vault_owner.classic_address).await;

        // The depositor is a domain member, so the deposit needs no credentials.
        let mut deposit = VaultDeposit {
            common_fields: CommonFields {
                account: depositor.classic_address.clone().into(),
                transaction_type: TransactionType::VaultDeposit,
                ..Default::default()
            },
            vault_id: vault_id.clone().into(),
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            ..Default::default()
        };
        test_transaction(&mut deposit, &depositor).await;

        // The withdrawal presents the credential that proves membership.
        let mut withdraw = VaultWithdraw {
            common_fields: CommonFields {
                account: depositor.classic_address.clone().into(),
                transaction_type: TransactionType::VaultWithdraw,
                ..Default::default()
            },
            vault_id: vault_id.clone().into(),
            amount: Amount::XRPAmount(XRPAmount::from("500000")),
            ..Default::default()
        }
        .with_credential_ids(vec![credential_id.clone().into()]);
        test_transaction(&mut withdraw, &depositor).await;

        let objects = account_objects_json(&vault_owner.classic_address).await;
        let vault = &objects["account_objects"][0];
        assert_eq!(
            vault["AssetsTotal"].as_str().unwrap_or("0"),
            "500000",
            "half of the deposit should remain in the vault"
        );
    })
    .await
}
