//! Shared helpers for XLS-65 SingleAssetVault integration tests.
//!
//! Centralises the repeated `account_objects` → vault query pattern so that
//! `vault_create`, `vault_clawback`, `vault_info`, and `ledger_entry` tests
//! all use a single authoritative copy.

#![allow(dead_code)]

use serde_json::Value;

use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::{CommonFields as ReqCommonFields, RequestMethod};

#[cfg(feature = "std")]
use xrpl::asynch::clients::XRPLAsyncClient;

/// Build an `AccountObjects` request that filters for `Vault` entries owned by `owner`.
pub fn vault_ao_request(owner: &str) -> AccountObjects<'_> {
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
}

/// Fetch all vault `account_objects` for `owner` as raw JSON.
#[cfg(feature = "std")]
pub async fn account_objects_json(owner: &str) -> Value {
    let client = super::get_client().await;
    let resp = client
        .request(vault_ao_request(owner).into())
        .await
        .expect("account_objects request failed");
    resp.raw_result.expect(
        "account_objects response contained no raw_result — server may have returned an error",
    )
}

/// Return the ledger object ID (`index`) of the first vault owned by `owner`.
///
/// Panics if no vault is found.
#[cfg(feature = "std")]
pub async fn get_vault_id(owner: &str) -> String {
    let resp = account_objects_json(owner).await;
    let objects = resp["account_objects"]
        .as_array()
        .expect("account_objects array missing");
    assert!(!objects.is_empty(), "no vault found for {owner}");
    objects[0]["index"]
        .as_str()
        .expect("vault index missing")
        .to_string()
}

/// Return both the ledger object ID and the `Sequence` of the first vault owned by `owner`.
///
/// Panics if no vault is found or if either field is missing.
#[cfg(feature = "std")]
pub async fn get_vault_id_and_seq(owner: &str) -> (String, u32) {
    let resp = account_objects_json(owner).await;
    let objects = resp["account_objects"]
        .as_array()
        .expect("account_objects array missing");
    assert!(!objects.is_empty(), "no vault found for {owner}");
    let vault_id = objects[0]["index"]
        .as_str()
        .expect("vault index missing")
        .to_string();
    let seq_u64 = objects[0]["Sequence"]
        .as_u64()
        .expect("vault Sequence missing");
    let seq = u32::try_from(seq_u64)
        .expect("vault Sequence exceeds u32::MAX — unexpected protocol value");
    (vault_id, seq)
}

/// Return the `AssetsTotal` of the first vault owned by `owner`.
///
/// Panics if no vault exists — a missing vault must surface as a test failure,
/// not as a silent `"0"` that lets downstream assertions false-pass.
#[cfg(feature = "std")]
pub async fn vault_assets_total(owner: &str) -> String {
    let resp = account_objects_json(owner).await;
    let objects = resp["account_objects"]
        .as_array()
        .expect("account_objects array missing");
    assert!(
        !objects.is_empty(),
        "no vault found for {owner} — cannot read AssetsTotal"
    );
    objects[0]["AssetsTotal"]
        .as_str()
        .unwrap_or("0")
        .to_string()
}

/// Return the number of vault objects owned by `owner`.
#[cfg(feature = "std")]
pub async fn vault_count(owner: &str) -> usize {
    let resp = account_objects_json(owner).await;
    resp["account_objects"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0)
}
