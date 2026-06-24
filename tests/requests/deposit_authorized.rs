// Scenarios:
//   - base: send a deposit_authorized request between two funded wallets
//     and verify that deposits are authorized by default
//   - with_credentials: provision a real Credential object, pass its hash,
//     verify the response echoes it back

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock, CREDENTIAL_TYPE_KYC};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::deposit_authorize::DepositAuthorized;
use xrpl::models::results;
use xrpl::models::results::deposit_authorize::DepositAuthorized as DepositAuthorizedResult;
use xrpl::models::transactions::{
    credential_accept::CredentialAccept, credential_create::CredentialCreate, CommonFields,
    TransactionType,
};

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

const CREDENTIAL_TYPE: &str = CREDENTIAL_TYPE_KYC;

#[tokio::test]
async fn test_deposit_authorized_with_credentials_echoed_in_response() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        // Step 1: issuer creates a credential for subject.
        let mut create = CredentialCreate {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::CredentialCreate,
                ..Default::default()
            },
            subject: subject.classic_address.clone().into(),
            credential_type: CREDENTIAL_TYPE.into(),
            ..Default::default()
        };
        test_transaction(&mut create, &issuer).await;

        // Step 2: subject accepts the credential.
        let mut accept = CredentialAccept {
            common_fields: CommonFields {
                account: subject.classic_address.clone().into(),
                transaction_type: TransactionType::CredentialAccept,
                ..Default::default()
            },
            issuer: issuer.classic_address.clone().into(),
            credential_type: CREDENTIAL_TYPE.into(),
        };
        test_transaction(&mut accept, &subject).await;

        // Step 3: read the on-chain credential hash from account_objects.
        let ao_resp = client
            .request(
                AccountObjects::new(
                    None,
                    subject.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::Credential),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects request failed");
        let ao_result: results::account_objects::AccountObjects<'_> =
            ao_resp.try_into().expect("parse account_objects");
        assert!(
            !ao_result.account_objects.is_empty(),
            "credential object should exist after accept"
        );
        let credential_hash = ao_result.account_objects[0]["index"]
            .as_str()
            .expect("index field missing on credential object")
            .to_string();

        // Step 4: query deposit_authorized passing the real credential hash.
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
