// XLS-65 SingleAssetVault — vault_info RPC integration tests
//
// Scenarios:
//   - by_vault_id:    VaultCreate → vault_info(vault_id) → assert vault object
//   - by_owner_seq:  same vault → vault_info(owner, seq) → assert same vault object
//
// Requires an XLS-65-enabled xrpld node (3.2.0+) at localhost:5005.

#[cfg(feature = "integration")]
mod tests {
    use crate::common::{
        generate_funded_wallet, get_client, test_transaction, with_blockchain_lock,
    };
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
    use xrpl::models::requests::vault_info::VaultInfo;
    use xrpl::models::requests::{CommonFields as ReqCommonFields, RequestMethod};
    use xrpl::models::results::vault_info::VaultInfo as VaultInfoResult;
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::{Currency, IssuedCurrency};

    async fn create_iou_vault(
        vault_owner: &xrpl::wallet::Wallet,
        issuer_address: &str,
    ) -> (String, u32) {
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                issuer_address.into(),
            )),
            withdrawal_policy: Some(1),
            ..Default::default()
        };
        test_transaction(&mut vault_create, vault_owner).await;

        let client = get_client().await;
        let resp = client
            .request(
                AccountObjects {
                    common_fields: ReqCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: vault_owner.classic_address.as_str().into(),
                    r#type: Some(AccountObjectType::Vault),
                    ledger_lookup: None,
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let raw = resp.raw_result.unwrap_or(serde_json::Value::Null);
        let objects = raw["account_objects"]
            .as_array()
            .expect("account_objects array missing");
        assert!(!objects.is_empty(), "no vault found after VaultCreate");

        let vault_id = objects[0]["index"]
            .as_str()
            .expect("vault index missing")
            .to_string();
        let seq = objects[0]["Sequence"]
            .as_u64()
            .expect("vault Sequence missing") as u32;

        (vault_id, seq)
    }

    #[tokio::test]
    async fn test_vault_info_by_vault_id() {
        with_blockchain_lock(|| async {
            let issuer = generate_funded_wallet().await;
            let vault_owner = generate_funded_wallet().await;

            let (vault_id, _) = create_iou_vault(&vault_owner, &issuer.classic_address).await;

            let req = VaultInfo::new(None, vault_id.as_str().into(), None, None);
            let client = get_client().await;
            let resp = client
                .request(req.into())
                .await
                .expect("vault_info by vault_id failed");

            let result: VaultInfoResult =
                resp.try_into().expect("failed to parse vault_info result");

            let vault = result.vault.expect("vault field missing in response");
            assert_eq!(
                vault["LedgerEntryType"].as_str(),
                Some("Vault"),
                "unexpected LedgerEntryType"
            );
            assert_eq!(
                vault["Owner"].as_str(),
                Some(vault_owner.classic_address.as_str()),
                "vault Owner mismatch"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_vault_info_by_owner_seq() {
        with_blockchain_lock(|| async {
            let issuer = generate_funded_wallet().await;
            let vault_owner = generate_funded_wallet().await;

            let (vault_id, seq) = create_iou_vault(&vault_owner, &issuer.classic_address).await;

            let req = VaultInfo::new_by_owner(
                None,
                vault_owner.classic_address.as_str().into(),
                seq,
                None,
                None,
            );
            let client = get_client().await;
            let resp = client
                .request(req.into())
                .await
                .expect("vault_info by owner+seq failed");

            let result: VaultInfoResult =
                resp.try_into().expect("failed to parse vault_info result");

            let vault = result.vault.expect("vault field missing in response");
            assert_eq!(
                vault["LedgerEntryType"].as_str(),
                Some("Vault"),
                "unexpected LedgerEntryType"
            );
            // Both lookup modes should return the same vault object ID
            assert_eq!(
                vault["index"].as_str(),
                Some(vault_id.as_str()),
                "vault index mismatch between vault_id and owner+seq lookups"
            );
        })
        .await;
    }
}
