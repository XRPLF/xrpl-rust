// xrpl.js reference: packages/xrpl/test/integration/transactions/permissionedDomain.test.ts
// rippled reference: src/test/app/PermissionedDomains_test.cpp
//
// Scenarios:
//   - base: create a new PermissionedDomain, verify tesSUCCESS
//   - account_objects_filter: filter by type=permissioned_domain; verify 1 object, Flags=0,
//     non-empty AcceptedCredentials
//   - ledger_entry_by_index: query domain by ledger hash; deep-equal vs account_objects node
//   - ledger_entry_by_account_seq: query domain by owner account + sequence
//   - update: replace credentials on existing domain (KYC → AML), verify KYC absent
//   - delete: full lifecycle (Set → account_objects → Delete → verify gone)

use crate::common::{
    constants::STANDALONE_URL, generate_funded_wallet, get_client, ledger_accept,
    with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::{CommonFields as RequestCommonFields, RequestMethod};
use xrpl::models::results;
use xrpl::models::transactions::permissioned_domain_delete::PermissionedDomainDelete;
use xrpl::models::transactions::permissioned_domain_set::PermissionedDomainSet;
use xrpl::models::transactions::{CommonFields, Credential, TransactionType};

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn kyc_credential(issuer: &str) -> Credential {
    Credential {
        issuer: issuer.to_string(),
        credential_type: "4B5943".to_string(), // hex("KYC")
    }
}

fn aml_credential(issuer: &str) -> Credential {
    Credential {
        issuer: issuer.to_string(),
        credential_type: "414D4C".to_string(), // hex("AML")
    }
}

fn new_pd_set(
    account: &str,
    domain_id: Option<String>,
    credentials: Vec<Credential>,
) -> PermissionedDomainSet<'static> {
    PermissionedDomainSet {
        common_fields: CommonFields {
            account: account.to_string().into(),
            transaction_type: TransactionType::PermissionedDomainSet,
            ..Default::default()
        },
        domain_id: domain_id.map(|s| s.into()),
        accepted_credentials: credentials,
    }
}

/// Raw `ledger_entry` RPC call — bypasses the typed XRPLClient whose serde only handles
/// AccountRoot. Returns the `result` object from the response.
async fn rpc_ledger_entry(params: serde_json::Value) -> serde_json::Value {
    let body = serde_json::json!({"method": "ledger_entry", "params": [params]});
    let resp = reqwest::Client::new()
        .post(STANDALONE_URL)
        .json(&body)
        .send()
        .await
        .expect("ledger_entry RPC request failed")
        .json::<serde_json::Value>()
        .await
        .expect("ledger_entry RPC response parse failed");
    resp["result"].clone()
}

async fn ledger_entry_by_index(domain_id: &str) -> serde_json::Value {
    rpc_ledger_entry(serde_json::json!({"permissioned_domain": domain_id})).await
}

async fn ledger_entry_by_account_seq(account: &str, seq: u64) -> serde_json::Value {
    rpc_ledger_entry(serde_json::json!({
        "permissioned_domain": {"account": account, "seq": seq}
    }))
    .await
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_permissioned_domain_set_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");

        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;
    })
    .await;
}

/// Mirrors xrpl.js step-2: account_objects filter, Flags=0, AcceptedCredentials non-empty.
#[tokio::test]
async fn test_permissioned_domain_account_objects_filter() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );
        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");

        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let ao: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("account_objects parse failed");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Expected exactly 1 PermissionedDomain object, got {}",
            ao.account_objects.len()
        );

        let obj = &ao.account_objects[0];
        assert_eq!(obj["LedgerEntryType"], "PermissionedDomain");
        assert_eq!(obj["Owner"], wallet.classic_address.as_str());

        // xrpl.js parity: newly-created PD has Flags = 0
        assert_eq!(
            obj["Flags"].as_u64().unwrap_or(u64::MAX),
            0,
            "PermissionedDomain Flags should be 0"
        );

        let creds = obj["AcceptedCredentials"]
            .as_array()
            .expect("AcceptedCredentials must be an array");
        assert!(
            !creds.is_empty(),
            "AcceptedCredentials must not be empty on a valid domain"
        );
    })
    .await;
}

/// Mirrors xrpl.js step-3: ledger_entry by index deep-equals the account_objects node.
#[tokio::test]
async fn test_permissioned_domain_ledger_entry_by_index() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );
        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");

        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Fetch domain via account_objects
        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects failed");
        let ao: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("account_objects parse");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Expected 1 PermissionedDomain, got {}",
            ao.account_objects.len()
        );
        let pd_obj = &ao.account_objects[0];
        let domain_id = pd_obj["index"]
            .as_str()
            .or_else(|| pd_obj["LedgerIndex"].as_str())
            .expect("index/LedgerIndex field missing on account_objects[0]")
            .to_string();

        // Fetch same domain via ledger_entry by index
        let entry = ledger_entry_by_index(&domain_id).await;

        assert_eq!(
            entry["node"]["LedgerEntryType"], "PermissionedDomain",
            "Expected PermissionedDomain, got: {}",
            entry["node"]["LedgerEntryType"]
        );
        assert_eq!(
            entry["node"]["Owner"],
            wallet.classic_address.as_str(),
            "Owner mismatch"
        );

        // xrpl.js parity: ledger_entry node must deep-equal the account_objects entry
        assert_eq!(
            &entry["node"], pd_obj,
            "ledger_entry node must match account_objects[0]"
        );
    })
    .await;
}

#[tokio::test]
async fn test_permissioned_domain_ledger_entry_by_account_seq() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );
        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");

        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Get sequence from account_objects
        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects failed");
        let ao: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("account_objects parse");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Expected 1 PermissionedDomain, got {}",
            ao.account_objects.len()
        );
        let seq = ao.account_objects[0]["Sequence"]
            .as_u64()
            .expect("Sequence field missing on PermissionedDomain");

        // Query ledger_entry by account + sequence (raw RPC)
        let entry = ledger_entry_by_account_seq(&wallet.classic_address, seq).await;

        assert_eq!(
            entry["node"]["LedgerEntryType"], "PermissionedDomain",
            "Expected PermissionedDomain, got: {}",
            entry["node"]["LedgerEntryType"]
        );
        assert_eq!(
            entry["node"]["Sequence"]
                .as_u64()
                .expect("Sequence missing"),
            seq,
            "Sequence mismatch"
        );
    })
    .await;
}

#[tokio::test]
async fn test_permissioned_domain_update_credentials() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // Step 1: create with KYC credential
        let mut create_tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );
        let result = sign_and_submit(&mut create_tx, client, &wallet, true, true)
            .await
            .expect("create PDSet failed");

        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Get domain_id
        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects failed");
        let ao: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("account_objects parse");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Expected 1 PermissionedDomain, got {}",
            ao.account_objects.len()
        );
        let domain_id = ao.account_objects[0]["index"]
            .as_str()
            .or_else(|| ao.account_objects[0]["LedgerIndex"].as_str())
            .expect("index/LedgerIndex field missing on account_objects[0]")
            .to_string();

        // Step 2: update with AML credential (replacing KYC)
        let mut update_tx = new_pd_set(
            &wallet.classic_address,
            Some(domain_id.clone()),
            vec![aml_credential(&wallet.classic_address)],
        );
        let update_result = sign_and_submit(&mut update_tx, client, &wallet, true, true)
            .await
            .expect("update PDSet failed");

        assert_eq!(
            update_result.engine_result, "tesSUCCESS",
            "PDSet update should succeed: {}",
            update_result.engine_result
        );
        ledger_accept().await;

        // Step 3: verify AML present and KYC absent (raw RPC)
        let entry = ledger_entry_by_index(&domain_id).await;

        let creds = entry["node"]["AcceptedCredentials"]
            .as_array()
            .expect("AcceptedCredentials must be an array");

        assert_eq!(creds.len(), 1, "Expected exactly 1 credential after update");

        let cred_type = creds[0]["Credential"]["CredentialType"]
            .as_str()
            .expect("CredentialType field missing in AcceptedCredentials[0].Credential")
            .to_uppercase();

        assert_eq!(
            cred_type,
            "414D4C", // hex("AML")
            "Expected AML credential type after update, got: {}",
            cred_type
        );
        assert!(
            !creds.iter().any(|c| {
                c["Credential"]["CredentialType"]
                    .as_str()
                    .unwrap_or("")
                    .eq_ignore_ascii_case("4B5943")
            }),
            "KYC credential should be absent after update to AML"
        );
    })
    .await;
}

/// Full lifecycle: PDSet → account_objects (verify 1) → PDDelete → account_objects (verify 0).
/// Mirrors xrpl.js step-3 (delete) in permissionedDomain.test.ts.
#[tokio::test]
async fn test_permissioned_domain_delete_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // Create the domain first
        let mut set_tx = new_pd_set(
            &wallet.classic_address,
            None,
            vec![kyc_credential(&wallet.classic_address)],
        );
        let set_result = sign_and_submit(&mut set_tx, client, &wallet, true, true)
            .await
            .expect("PermissionedDomainSet submission should not fail");

        assert_eq!(
            set_result.engine_result, "tesSUCCESS",
            "PermissionedDomainSet must succeed before delete test: {}",
            set_result.engine_result
        );
        ledger_accept().await;

        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request should succeed");
        let account_objects: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("account_objects response should deserialize");

        assert_eq!(
            account_objects.account_objects.len(),
            1,
            "Expected 1 PermissionedDomain before delete"
        );

        let domain_id = account_objects.account_objects[0]["index"]
            .as_str()
            .or_else(|| account_objects.account_objects[0]["LedgerIndex"].as_str())
            .expect("PermissionedDomain object should have an index field")
            .to_string();

        let mut delete_tx = PermissionedDomainDelete {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::PermissionedDomainDelete,
                ..Default::default()
            },
            domain_id: domain_id.into(),
        };

        let delete_result = sign_and_submit(&mut delete_tx, client, &wallet, true, true)
            .await
            .expect("PermissionedDomainDelete submission should not fail");

        assert_eq!(
            delete_result.engine_result, "tesSUCCESS",
            "PermissionedDomainDelete should succeed: {}",
            delete_result.engine_result
        );
        ledger_accept().await;

        // Verify domain is gone
        let ao_after = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::PermissionedDomain),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects after delete should succeed");
        let ao_result: results::account_objects::AccountObjects<'_> = ao_after
            .try_into()
            .expect("account_objects after delete should deserialize");

        assert_eq!(
            ao_result.account_objects.len(),
            0,
            "PermissionedDomain should be absent after deletion"
        );
    })
    .await;
}
