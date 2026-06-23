// xrpl.js reference: N/A (XLS-80 is a new feature)
// rippled reference: src/test/app/PermissionedDomains_test.cpp
//
// Scenarios:
//   - base: create a PermissionedDomain, delete it, verify it is gone from account_objects

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::permissioned_domain_delete::PermissionedDomainDelete;
use xrpl::models::transactions::permissioned_domain_set::PermissionedDomainSet;
use xrpl::models::transactions::{CommonFields, Credential, TransactionType};

#[tokio::test]
async fn test_permissioned_domain_delete_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut set_tx = PermissionedDomainSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::PermissionedDomainSet,
                ..Default::default()
            },
            domain_id: None,
            accepted_credentials: vec![Credential {
                issuer: wallet.classic_address.clone(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        };

        let set_result = sign_and_submit(&mut set_tx, client, &wallet, true, true)
            .await
            .expect("PermissionedDomainSet submission should not fail");

        if set_result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }

        assert_eq!(
            set_result.engine_result, "tesSUCCESS",
            "PermissionedDomainSet must succeed before delete test: {}",
            set_result.engine_result
        );
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
            .expect("account_objects request should succeed");
        let account_objects: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("account_objects response should deserialize");

        assert_eq!(
            account_objects.account_objects.len(),
            1,
            "Expected 1 PermissionedDomain before delete"
        );

        let domain_id = account_objects
            .account_objects
            .iter()
            .find(|o| o["LedgerEntryType"] == "PermissionedDomain")
            .and_then(|o| o["index"].as_str().or_else(|| o["LedgerIndex"].as_str()))
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
