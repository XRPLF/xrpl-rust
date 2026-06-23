// XLS-65 SingleAssetVault — vault_info RPC integration tests
//
// Mirrors xrpl.js packages/xrpl/test/integration/requests/vaultInfo.test.ts "base" scenario:
//   - XRP vault created, then queried by vault_id and by owner+seq
//   - Asserts vault object fields: LedgerEntryType, Owner, Asset, WithdrawalPolicy,
//     AssetsTotal, AssetsAvailable, ShareMPTID, shares subobject
//   - Both lookup modes return the same vault index
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
    use xrpl::models::Currency;

    /// Create an XRP vault and return (vault_id, vault_sequence).
    async fn create_xrp_vault(vault_owner: &xrpl::wallet::Wallet) -> (String, u32) {
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: vault_owner.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::default(), // XRP
            withdrawal_policy: Some(1),
            assets_maximum: Some("1000000000".into()),
            data: Some(hex::encode("vault metadata").to_uppercase().into()),
            mptoken_metadata: Some(hex::encode("share metadata").to_uppercase().into()),
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
        // Vault object carries the Sequence of the VaultCreate tx directly.
        let seq = objects[0]["Sequence"]
            .as_u64()
            .expect("vault Sequence missing") as u32;

        (vault_id, seq)
    }

    #[tokio::test]
    async fn test_vault_info_by_vault_id() {
        with_blockchain_lock(|| async {
            let vault_owner = generate_funded_wallet().await;
            let (vault_id, _) = create_xrp_vault(&vault_owner).await;

            let req = VaultInfo::new(None, vault_id.as_str().into(), None, None);
            let client = get_client().await;
            let resp = client
                .request(req.into())
                .await
                .expect("vault_info by vault_id failed");

            let result: VaultInfoResult =
                resp.try_into().expect("failed to parse vault_info result");

            // ledger_current_index must be a number when present (open-ledger mode)
            if let Some(idx) = result.ledger_current_index {
                assert!(idx > 0, "ledger_current_index should be positive");
            }

            let vault = result.vault.expect("vault field missing in response");

            assert_eq!(vault["LedgerEntryType"].as_str(), Some("Vault"));
            assert_eq!(
                vault["Owner"].as_str(),
                Some(vault_owner.classic_address.as_str()),
                "vault Owner mismatch"
            );
            // XRP asset serializes as {"currency":"XRP"}
            assert_eq!(vault["Asset"]["currency"].as_str(), Some("XRP"));
            assert_eq!(vault["WithdrawalPolicy"].as_u64(), Some(1));
            assert_eq!(
                vault["AssetsTotal"].as_str().unwrap_or("0"),
                "0",
                "new vault should have zero AssetsTotal"
            );
            assert_eq!(
                vault["AssetsAvailable"].as_str().unwrap_or("0"),
                "0",
                "new vault should have zero AssetsAvailable"
            );

            // shares sub-object
            let shares = &vault["shares"];
            assert!(
                !shares.is_null(),
                "shares sub-object should be present in vault_info response"
            );
            assert_eq!(shares["LedgerEntryType"].as_str(), Some("MPTokenIssuance"));
            assert_eq!(
                shares["OutstandingAmount"].as_str().unwrap_or("0"),
                "0",
                "new vault shares outstanding should be zero"
            );
            // ShareMPTID on vault should match mpt_issuance_id on shares
            let share_mpt_id = vault["ShareMPTID"].as_str().unwrap_or("");
            let mpt_issuance_id = shares["mpt_issuance_id"].as_str().unwrap_or("");
            assert!(!share_mpt_id.is_empty(), "ShareMPTID should be present");
            assert_eq!(
                share_mpt_id, mpt_issuance_id,
                "ShareMPTID should match shares.mpt_issuance_id"
            );
            // shares.Issuer should match vault Account (pseudo-account)
            assert_eq!(
                shares["Issuer"].as_str(),
                vault["Account"].as_str(),
                "shares.Issuer should match vault Account"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_vault_info_by_owner_seq() {
        with_blockchain_lock(|| async {
            let vault_owner = generate_funded_wallet().await;
            let (vault_id, seq) = create_xrp_vault(&vault_owner).await;

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
            assert_eq!(vault["LedgerEntryType"].as_str(), Some("Vault"));
            // Both lookup modes must return the same vault object
            assert_eq!(
                vault["index"].as_str(),
                Some(vault_id.as_str()),
                "owner+seq lookup must return same vault as vault_id lookup"
            );
        })
        .await;
    }
}
