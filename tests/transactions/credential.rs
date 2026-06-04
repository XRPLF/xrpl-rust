// Scenarios:
//   - create: issuer creates a credential for a subject
//   - accept: subject accepts the credential
//   - delete: issuer deletes the credential

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::{
    credential_accept::CredentialAccept, credential_create::CredentialCreate,
    credential_delete::CredentialDelete,
};

#[tokio::test]
async fn test_credential_create_accept_delete() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;
        let credential_type = "4B5943"; // "KYC"

        let mut create = CredentialCreate::new(
            issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            subject.classic_address.clone().into(),
            credential_type.into(),
            None,
            None,
        );
        test_transaction(&mut create, &issuer).await;

        let mut accept = CredentialAccept::new(
            subject.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            issuer.classic_address.clone().into(),
            credential_type.into(),
        );
        test_transaction(&mut accept, &subject).await;

        let mut delete = CredentialDelete::new(
            issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(subject.classic_address.clone().into()),
            None,
            credential_type.into(),
        );
        test_transaction(&mut delete, &issuer).await;
    })
    .await;
}
