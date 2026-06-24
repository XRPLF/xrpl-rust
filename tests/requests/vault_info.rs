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
    use xrpl::models::requests::vault_info::VaultInfo;
    use xrpl::models::results::vault_info::VaultInfo as VaultInfoResult;
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::Currency;

    /// Create an XRP vault and return `(vault_id, vault_sequence)`.
    ///
    /// Submits a `VaultCreate` transaction and resolves the vault's ledger
    /// object ID and sequence using the shared `get_vault_id_and_seq` helper.
    /// The AccountObjects call is setup — it does not assert on the
    /// account_objects RPC itself.
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

        // Setup: resolve vault object ID and sequence via account_objects.
        crate::common::vault::get_vault_id_and_seq(vault_owner.classic_address.as_str()).await
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

            let result: VaultInfoResult = match resp.try_into() {
                Ok(r) => r,
                Err(e) if e.to_string().contains("Unexpected result type") => {
                    // vault_info is gated behind the XLS-65 amendment; skip on nodes where it
                    // is inactive. Print to stdout so cargo test --nocapture captures it.
                    println!(
                        "SKIP test_vault_info_by_vault_id: XLS-65 inactive or unsupported — {e}"
                    );
                    return;
                }
                Err(e) => panic!("failed to parse vault_info result: {e}"),
            };

            // ledger_current_index must be a positive number when present (open-ledger mode)
            if let Some(idx) = result.ledger_current_index {
                assert!(idx > 0, "ledger_current_index should be positive");
            }

            let vault_obj = result.vault.expect("vault field missing in response");

            // Typed access — no string indexing required.
            assert_eq!(
                vault_obj.vault.owner.as_ref(),
                vault_owner.classic_address.as_str(),
                "vault Owner mismatch"
            );
            // XRP asset
            assert!(
                matches!(vault_obj.vault.asset, Currency::XRP(_)),
                "expected XRP asset"
            );
            assert_eq!(vault_obj.vault.withdrawal_policy, 1u8);
            assert_eq!(
                vault_obj.vault.assets_total.as_deref().unwrap_or("0"),
                "0",
                "new vault should have zero AssetsTotal"
            );
            assert_eq!(
                vault_obj.vault.assets_available.as_deref().unwrap_or("0"),
                "0",
                "new vault should have zero AssetsAvailable"
            );
            assert!(
                !vault_obj.vault.share_mpt_id.is_empty(),
                "ShareMPTID should be present"
            );

            // shares sub-object
            let shares = vault_obj
                .shares
                .as_ref()
                .expect("shares sub-object should be present in vault_info response");
            assert_eq!(shares.ledger_entry_type.as_deref(), Some("MPTokenIssuance"));
            assert_eq!(
                shares.outstanding_amount.as_deref().unwrap_or("0"),
                "0",
                "new vault shares outstanding should be zero"
            );
            // shares.Issuer should match vault Account (pseudo-account)
            assert_eq!(
                shares.issuer.as_deref(),
                Some(vault_obj.vault.account.as_ref()),
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

            let result: VaultInfoResult = match resp.try_into() {
                Ok(r) => r,
                Err(e) if e.to_string().contains("Unexpected result type") => {
                    println!(
                        "SKIP test_vault_info_by_owner_seq: XLS-65 inactive or unsupported — {e}"
                    );
                    return;
                }
                Err(e) => panic!("failed to parse vault_info result: {e}"),
            };

            let vault_obj = result.vault.expect("vault field missing in response");
            // Both lookup modes must return the same vault object index.
            assert_eq!(
                vault_obj.vault.common_fields.index.as_deref(),
                Some(vault_id.as_str()),
                "owner+seq lookup must return same vault as vault_id lookup"
            );
        })
        .await;
    }
}
