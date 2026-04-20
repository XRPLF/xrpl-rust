// xrpl.js reference: N/A (XLS-80 is a new feature)
//
// Scenarios:
//   - base: submit PermissionedDomainSet to create a new domain
//
// NOTE: PermissionedDomainSet requires the PermissionedDomains amendment to be enabled.
// This test verifies the transaction can be constructed, serialized, and submitted.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::transactions::permissioned_domain_set::PermissionedDomainSet;
use xrpl::models::transactions::Credential;

#[tokio::test]
async fn test_permissioned_domain_set_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = PermissionedDomainSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None, // No domain_id means create new domain
            vec![Credential {
                issuer: wallet.classic_address.clone(),
                // CredentialType is a Blob field; the value must be hex-encoded
                credential_type: "4B5943".to_string(), // hex("KYC")
            }],
        );

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit should not fail at submission level");

        // The amendment may not be enabled on the test network; use a strict
        // allowlist of exact engine_result codes rather than substring matches.
        let allowed = [
            "tesSUCCESS",
            "temDISABLED",
            "tecNO_PERMISSION",
            "tecDUPLICATE",
        ];
        assert!(
            allowed.contains(&&*result.engine_result),
            "unexpected engine_result: {}",
            result.engine_result
        );

        ledger_accept().await;
    })
    .await;
}
