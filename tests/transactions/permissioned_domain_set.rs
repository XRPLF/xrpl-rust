// xrpl.js reference: N/A (no dedicated PD integration test in xrpl.js yet)
// rippled reference: src/test/app/PermissionedDomains_test.cpp
//
// Scenarios:
//   - base: create a new PermissionedDomain, verify tesSUCCESS
//   - account_objects_filter: filter by type=permissioned_domain; verify exactly 1 object
//   - ledger_entry_by_index: query domain by its ledger hash
//   - ledger_entry_by_account_seq: query domain by owner account + sequence
//   - update: replace credentials on existing domain (KYC → AML), verify KYC absent

use crate::common::{
    constants::STANDALONE_URL, generate_funded_wallet, get_client, ledger_accept,
    with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::permissioned_domain_set::PermissionedDomainSet;
use xrpl::models::transactions::{CommonFields, Credential, TransactionType};

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

/// Query ledger_entry via raw RPC (bypasses typed client serde which only handles AccountRoot).
async fn ledger_entry_by_index(domain_id: &str) -> serde_json::Value {
    let body = serde_json::json!({
        "method": "ledger_entry",
        "params": [{"permissioned_domain": domain_id}]
    });
    let resp = reqwest::Client::new()
        .post(STANDALONE_URL)
        .json(&body)
        .send()
        .await
        .expect("ledger_entry request failed")
        .json::<serde_json::Value>()
        .await
        .expect("ledger_entry response parse failed");
    resp["result"].clone()
}

/// Query ledger_entry by account + sequence via raw RPC.
async fn ledger_entry_by_account_seq(account: &str, seq: u64) -> serde_json::Value {
    let body = serde_json::json!({
        "method": "ledger_entry",
        "params": [{
            "permissioned_domain": {
                "account": account,
                "seq": seq
            }
        }]
    });
    let resp = reqwest::Client::new()
        .post(STANDALONE_URL)
        .json(&body)
        .send()
        .await
        .expect("ledger_entry request failed")
        .json::<serde_json::Value>()
        .await
        .expect("ledger_entry response parse failed");
    resp["result"].clone()
}

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

        let allowed = ["tesSUCCESS", "temDISABLED"];
        assert!(
            allowed.contains(&&*result.engine_result),
            "unexpected engine_result: {}",
            result.engine_result
        );
        ledger_accept().await;
    })
    .await;
}

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

        if result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }
        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
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
            .expect("account_objects request failed");

        let ao: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("account_objects parse failed");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Expected exactly 1 PermissionedDomain object, got {}",
            ao.account_objects.len()
        );

        let obj = &ao.account_objects[0];
        assert_eq!(obj["LedgerEntryType"], "PermissionedDomain");
        assert_eq!(obj["Owner"], wallet.classic_address.as_str());

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

        if result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }
        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Get domain_id from account_objects
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
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

        // Query ledger_entry by index hash (raw RPC — typed client only handles AccountRoot)
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

        if result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }
        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Get sequence from account_objects
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
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
            entry["node"]["Sequence"].as_u64().expect("Sequence missing"),
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

        if result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }
        assert_eq!(result.engine_result, "tesSUCCESS");
        ledger_accept().await;

        // Get domain_id
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
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
            cred_type, "414D4C", // hex("AML")
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
