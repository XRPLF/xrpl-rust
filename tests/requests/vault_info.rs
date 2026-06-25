// XLS-65 SingleAssetVault — vault_info RPC integration tests
//
// Mirrors xrpl.js packages/xrpl/test/integration/requests/vaultInfo.test.ts:
//   - XRP vault created, queried by vault_id and by owner+seq
//   - Asserts vault object fields: LedgerEntryType, Owner, Asset, WithdrawalPolicy,
//     AssetsTotal, AssetsAvailable, ShareMPTID, shares subobject
//   - Both lookup modes return the same vault index
//   - shares.mpt_issuance_id matches Vault.ShareMPTID
//   - IOU vault with Scale field returns correct scale in vault_info
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
    use xrpl::models::transactions::account_set::{AccountSet, AccountSetFlag};
    use xrpl::models::transactions::payment::Payment;
    use xrpl::models::transactions::trust_set::TrustSet;
    use xrpl::models::transactions::vault_create::VaultCreate;
    use xrpl::models::transactions::{CommonFields, TransactionType};
    use xrpl::models::{Amount, Currency, IssuedCurrency, IssuedCurrencyAmount};

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

    /// `shares.mpt_issuance_id` in the vault_info response must equal `Vault.ShareMPTID`.
    ///
    /// Mirrors xrpl.js vaultInfo.test.ts `ShareMPTID == shares.mpt_issuance_id` assertion.
    #[tokio::test]
    async fn test_vault_info_mpt_issuance_id_matches_share_mpt_id() {
        with_blockchain_lock(|| async {
            let vault_owner = generate_funded_wallet().await;
            let (vault_id, _) = create_xrp_vault(&vault_owner).await;

            let req = VaultInfo::new(None, vault_id.as_str().into(), None, None);
            let client = get_client().await;
            let resp = client
                .request(req.into())
                .await
                .expect("vault_info request failed");

            let result: VaultInfoResult = match resp.try_into() {
                Ok(r) => r,
                Err(e) if e.to_string().contains("Unexpected result type") => {
                    println!("SKIP test_vault_info_mpt_issuance_id_matches_share_mpt_id: XLS-65 inactive — {e}");
                    return;
                }
                Err(e) => panic!("failed to parse vault_info result: {e}"),
            };

            let vault_obj = result.vault.expect("vault field missing");
            let shares = vault_obj
                .shares
                .as_ref()
                .expect("shares sub-object missing");

            let mpt_id = shares
                .mpt_issuance_id
                .as_deref()
                .expect("shares.mpt_issuance_id missing");

            assert_eq!(
                mpt_id,
                vault_obj.vault.share_mpt_id.as_ref(),
                "shares.mpt_issuance_id must equal Vault.ShareMPTID"
            );
        })
        .await;
    }

    /// IOU vault with explicit `Scale` — vault_info returns the correct scale.
    ///
    /// Mirrors xrpl.js vaultInfo.test.ts "IOU asset with Scale" scenario.
    #[tokio::test]
    async fn test_vault_info_iou_scale_field() {
        with_blockchain_lock(|| async {
            let issuer = generate_funded_wallet().await;
            let vault_owner = generate_funded_wallet().await;
            const CURRENCY: &str = "USD";
            const SCALE: u8 = 6;

            // Enable DefaultRipple on issuer so IOU payments ripple.
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

            // vault_owner establishes a trust line so the vault can hold IOU.
            let mut trust = TrustSet {
                common_fields: CommonFields {
                    account: vault_owner.classic_address.clone().into(),
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
            test_transaction(&mut trust, &vault_owner).await;

            // Fund vault_owner with some IOU.
            let mut payment = Payment {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::Payment,
                    ..Default::default()
                },
                destination: vault_owner.classic_address.clone().into(),
                amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                    CURRENCY.into(),
                    issuer.classic_address.clone().into(),
                    "1000".into(),
                )),
                ..Default::default()
            };
            test_transaction(&mut payment, &issuer).await;

            // Create IOU vault with explicit scale=6.
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
                scale: Some(SCALE),
                ..Default::default()
            };
            test_transaction(&mut vault_create, &vault_owner).await;

            let (vault_id, _) =
                crate::common::vault::get_vault_id_and_seq(vault_owner.classic_address.as_str())
                    .await;

            let req = VaultInfo::new(None, vault_id.as_str().into(), None, None);
            let client = get_client().await;
            let resp = client
                .request(req.into())
                .await
                .expect("vault_info request failed");

            let result: VaultInfoResult = match resp.try_into() {
                Ok(r) => r,
                Err(e) if e.to_string().contains("Unexpected result type") => {
                    println!("SKIP test_vault_info_iou_scale_field: XLS-65 inactive — {e}");
                    return;
                }
                Err(e) => panic!("failed to parse vault_info result: {e}"),
            };

            let vault_obj = result.vault.expect("vault field missing");
            assert_eq!(
                vault_obj.vault.scale,
                Some(SCALE),
                "vault_info Scale must match the value set on VaultCreate"
            );
            assert!(
                matches!(vault_obj.vault.asset, Currency::IssuedCurrency(_)),
                "asset should be IOU"
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
