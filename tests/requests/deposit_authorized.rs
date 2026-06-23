// Scenarios:
//   - base: send a deposit_authorized request between two funded wallets
//     and verify that deposits are authorized by default
//   - with_credentials: pass credentials param, verify the response echoes it back
//     (requires Credentials amendment enabled — marked #[ignore])

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::deposit_authorize::DepositAuthorized;
use xrpl::models::results::deposit_authorize::DepositAuthorized as DepositAuthorizedResult;

#[tokio::test]
async fn test_deposit_authorized_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet1 = crate::common::generate_funded_wallet().await;
        let wallet2 = crate::common::generate_funded_wallet().await;

        let request = DepositAuthorized::new(
            None,                                   // id
            wallet2.classic_address.clone().into(), // destination_account
            wallet1.classic_address.clone().into(), // source_account
            None,                                   // ledger_hash
            None,                                   // ledger_index
        );

        let response = client
            .request(request.into())
            .await
            .expect("deposit_authorized request failed");

        let result: DepositAuthorizedResult = response
            .try_into()
            .expect("failed to parse deposit_authorized result");

        // Verify deposit is authorized (default state)
        assert!(result.deposit_authorized);
        // Verify source and destination accounts match
        assert_eq!(
            result.source_account.as_ref(),
            wallet1.classic_address.as_str()
        );
        assert_eq!(
            result.destination_account.as_ref(),
            wallet2.classic_address.as_str()
        );
        // Verify ledger_current_index exists
        assert!(result.ledger_current_index.unwrap() > 0);
    })
    .await;
}

// ── with credentials: verify the response echoes the credentials field ────
//
// Requires: standalone rippled with Credentials amendment enabled.
// The credentials param must reference an accepted Credential ledger object
// owned by the source account; without it rippled returns an error instead.
// Remove #[ignore] once CI provisions credentials and enables the amendment.

#[tokio::test]
#[ignore = "requires Credentials amendment enabled and provisioned credential object in standalone rippled"]
async fn test_deposit_authorized_with_credentials_echoed_in_response() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let wallet1 = crate::common::generate_funded_wallet().await;
        let wallet2 = crate::common::generate_funded_wallet().await;

        // In a full test: provision a credential for wallet1, then pass its
        // hash in credentials. For now this test documents the expected shape.
        let credential_id = "A182EFBD154C9E80195082F86C1C8952FC0760A654B886F61BB0A59803B4387B";

        let request = DepositAuthorized::new(
            None,
            wallet2.classic_address.clone().into(),
            wallet1.classic_address.clone().into(),
            None,
            None,
        )
        .with_credentials(vec![credential_id.into()]);

        let response = client
            .request(request.into())
            .await
            .expect("deposit_authorized request failed");

        let result: DepositAuthorizedResult = response
            .try_into()
            .expect("failed to parse deposit_authorized result");

        // The rippled response must echo the credentials field (Protocol.h verified).
        let echoed = result
            .credentials
            .expect("credentials should be echoed in response");
        assert_eq!(echoed.len(), 1);
        assert_eq!(echoed[0].as_ref(), credential_id);
    })
    .await;
}
