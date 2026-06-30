// Scenarios:
//   - base: get an entry index from ledger_data, then query ledger_entry with that index
//   - vault_by_id: create a vault, fetch via VaultIdentifier::Id
//   - vault_by_owner_seq: same vault, fetch via VaultIdentifier::OwnerSeq

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::{
        ledger_data::LedgerData as LedgerDataRequest,
        ledger_entry::{LedgerEntry, VaultIdentifier},
        LedgerIndex,
    },
    results::ledger_data::LedgerData as LedgerDataResult,
    results::ledger_entry::LedgerEntry as LedgerEntryResult,
    transactions::{CommonFields, TransactionType},
    Currency,
};

#[tokio::test]
async fn test_ledger_entry_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        // First, get a valid entry index from ledger_data
        let data_request = LedgerDataRequest::new(
            None,                                       // id
            None,                                       // binary
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(1),                                    // limit
            None,                                       // marker
        );

        let data_response = client
            .request(data_request.into())
            .await
            .expect("ledger_data request failed");

        let data_result: LedgerDataResult = data_response
            .try_into()
            .expect("failed to parse ledger_data result");

        let entry_index = data_result.state[0].index.clone();

        // Now query ledger_entry with that index
        let entry_request = xrpl::models::requests::ledger_entry::LedgerEntry::new(
            None,                      // id
            None,                      // account_root
            None,                      // binary
            None,                      // check
            None,                      // deposit_preauth
            None,                      // directory
            None,                      // escrow
            Some(entry_index.clone()), // index
            None,                      // ledger_hash
            None,                      // ledger_index
            None,                      // offer
            None,                      // oracle
            None,                      // payment_channel
            None,                      // ripple_state
            None,                      // ticket
            None,                      // vault
        );

        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("failed ledger_entry request");

        let entry_result: xrpl::models::results::ledger_entry::LedgerEntry = entry_response
            .try_into()
            .expect("failed to parse ledger_entry result");

        // Verify the returned index matches what we requested
        assert_eq!(entry_result.index.as_ref(), entry_index.as_ref());
        // Verify the node is present (non-binary mode)
        assert!(entry_result.node.is_some());
    })
    .await;
}

/// Create an XRP vault and fetch it via `ledger_entry` using a direct vault ID.
///
/// Mirrors ckeshava's review request: "Can you add an integ test where the
/// ledger_entry RPC fetches a vault object?"
#[cfg(feature = "integration")]
#[tokio::test]
async fn test_ledger_entry_vault_by_id() {
    use xrpl::models::transactions::vault_create::VaultCreate;
    with_blockchain_lock(|| async {
        let wallet = crate::common::generate_funded_wallet().await;

        // Create an XRP vault.
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::default(), // XRP
            withdrawal_policy: Some(1),
            ..Default::default()
        };
        crate::common::test_transaction(&mut vault_create, &wallet).await;

        // Resolve the vault object ID via account_objects.
        let vault_id = crate::common::vault::get_vault_id(wallet.classic_address.as_str()).await;

        // Fetch vault via ledger_entry using a direct vault ID.
        let client = crate::common::get_client().await;
        let entry_request = LedgerEntry {
            vault: Some(VaultIdentifier::Id(vault_id.as_str().into())),
            ..Default::default()
        };
        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("ledger_entry vault by ID failed");

        let entry_result: LedgerEntryResult = entry_response
            .try_into()
            .expect("failed to parse ledger_entry vault result");

        assert!(
            entry_result.node.is_some(),
            "node should be present in ledger_entry response"
        );
        let node = entry_result.node.unwrap();
        assert_eq!(
            node["LedgerEntryType"].as_str(),
            Some("Vault"),
            "expected LedgerEntryType Vault"
        );
        assert_eq!(
            entry_result.index.as_ref(),
            vault_id.as_str(),
            "returned index must match requested vault_id"
        );
    })
    .await;
}

/// Create an XRP vault and fetch it via `ledger_entry` using owner + sequence.
///
/// Verifies that both `VaultIdentifier::Id` and `VaultIdentifier::OwnerSeq` resolve
/// to the same ledger object index.
#[cfg(feature = "integration")]
#[tokio::test]
async fn test_ledger_entry_vault_by_owner_seq() {
    use xrpl::models::transactions::vault_create::VaultCreate;
    with_blockchain_lock(|| async {
        let wallet = crate::common::generate_funded_wallet().await;

        // Create an XRP vault.
        let mut vault_create = VaultCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::VaultCreate,
                ..Default::default()
            },
            asset: Currency::default(), // XRP
            withdrawal_policy: Some(1),
            ..Default::default()
        };
        crate::common::test_transaction(&mut vault_create, &wallet).await;

        // Resolve the vault object ID and VaultCreate sequence.
        let (vault_id, seq) =
            crate::common::vault::get_vault_id_and_seq(wallet.classic_address.as_str()).await;

        // Fetch vault via ledger_entry using owner + sequence.
        let client = crate::common::get_client().await;
        let entry_request = LedgerEntry {
            vault: Some(VaultIdentifier::OwnerSeq {
                owner: wallet.classic_address.as_str().into(),
                seq,
            }),
            ..Default::default()
        };
        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("ledger_entry vault by owner+seq failed");

        let entry_result: LedgerEntryResult = entry_response
            .try_into()
            .expect("failed to parse ledger_entry vault owner+seq result");

        assert!(
            entry_result.node.is_some(),
            "node should be present in ledger_entry response"
        );
        // Both lookup modes must resolve to the same vault.
        assert_eq!(
            entry_result.index.as_ref(),
            vault_id.as_str(),
            "owner+seq lookup must return same vault as vault_id lookup"
        );
    })
    .await;
}
