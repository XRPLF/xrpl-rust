// Scenarios:
//   - base: send a deposit_authorized request between two funded wallets
//     and verify that deposits are authorized by default
//   - with_credentials: provision a real Credential object, pass its hash,
//     verify the response echoes it back

use crate::common::{
    generate_funded_wallet, provision_credential, with_blockchain_lock, CREDENTIAL_TYPE_KYC,
};
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

// ── with credentials: provision a real Credential, verify response echoes it ─

#[tokio::test]
async fn test_deposit_authorized_with_credentials_echoed_in_response() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let credential_hash = provision_credential(&issuer, &subject, CREDENTIAL_TYPE_KYC).await;

        let request = DepositAuthorized::new(
            None,
            destination.classic_address.clone().into(),
            subject.classic_address.clone().into(),
            None,
            None,
        )
        .with_credentials(vec![credential_hash.as_str().into()]);

        let response = client
            .request(request.into())
            .await
            .expect("deposit_authorized request failed");

        let result: DepositAuthorizedResult = response
            .try_into()
            .expect("failed to parse deposit_authorized result");

        // rippled echoes back the credentials field when it is supplied.
        let echoed = result
            .credentials
            .expect("credentials should be echoed in response");
        assert_eq!(echoed.len(), 1);
        assert_eq!(
            echoed[0].as_ref().to_uppercase(),
            credential_hash.to_uppercase()
        );
    })
    .await;
}
