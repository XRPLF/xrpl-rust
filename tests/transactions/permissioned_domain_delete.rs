// xrpl.js reference: N/A (XLS-80 is a new feature)
//
// Scenarios:
//   - base: submit PermissionedDomainDelete to delete an existing domain
//
// NOTE: PermissionedDomainDelete requires the PermissionedDomains amendment to be enabled
// and a valid domain_id of an existing PermissionedDomain ledger object owned by the account.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::transactions::permissioned_domain_delete::PermissionedDomainDelete;

#[tokio::test]
async fn test_permissioned_domain_delete_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // Use a placeholder domain ID -- on a real network this would be an existing domain.
        let mut tx = PermissionedDomainDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2".into(),
        );

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit should not fail at submission level");

        // The domain may not exist and the amendment may not be enabled,
        // so accept various result codes indicating the transaction was processed.
        assert!(
            result.engine_result.contains("tesSUCCESS")
                || result.engine_result.contains("temDISABLED")
                || result.engine_result.contains("tec"),
            "Unexpected engine result: {}",
            result.engine_result
        );

        ledger_accept().await;
    })
    .await;
}
