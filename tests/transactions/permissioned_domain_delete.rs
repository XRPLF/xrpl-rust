// xrpl.js reference: N/A (XLS-80 is a new feature)
//
// Scenarios:
//   - base: create a PermissionedDomain and delete that real ledger object
//
// NOTE: PermissionedDomainDelete requires the PermissionedDomains amendment to be enabled.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::requests::account_objects::AccountObjects;
use xrpl::models::results;
use xrpl::models::transactions::permissioned_domain_delete::PermissionedDomainDelete;
use xrpl::models::transactions::permissioned_domain_set::PermissionedDomainSet;
use xrpl::models::transactions::Credential;

#[tokio::test]
async fn test_permissioned_domain_delete_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut set_tx = PermissionedDomainSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![Credential {
                issuer: wallet.classic_address.clone(),
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        );

        let set_result = sign_and_submit(&mut set_tx, client, &wallet, true, true)
            .await
            .expect("PermissionedDomainSet submission should not fail");

        if set_result.engine_result == "temDISABLED" {
            ledger_accept().await;
            return;
        }

        assert_eq!(
            set_result.engine_result, "tesSUCCESS",
            "PermissionedDomainSet must create the domain before delete test: {}",
            set_result.engine_result
        );
        ledger_accept().await;

        let account_objects_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects request should succeed");
        let account_objects: results::account_objects::AccountObjects<'_> =
            account_objects_response
                .try_into()
                .expect("account_objects response should deserialize");

        let domain_id = account_objects
            .account_objects
            .iter()
            .find(|object| object["LedgerEntryType"] == "PermissionedDomain")
            .and_then(|object| {
                object["index"]
                    .as_str()
                    .or_else(|| object["LedgerIndex"].as_str())
            })
            .expect("created PermissionedDomain object should be returned by account_objects")
            .to_string();

        let mut delete_tx = PermissionedDomainDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            domain_id.into(),
        );

        let delete_result = sign_and_submit(&mut delete_tx, client, &wallet, true, true)
            .await
            .expect("PermissionedDomainDelete submission should not fail");

        assert_eq!(
            delete_result.engine_result, "tesSUCCESS",
            "PermissionedDomainDelete should delete the created domain: {}",
            delete_result.engine_result
        );

        ledger_accept().await;
    })
    .await;
}
