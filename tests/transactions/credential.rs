// Credential transaction integration tests.
//
// Scenarios:
//   1. create + accept + delete (full lifecycle by issuer)
//   2. self-issued credential (subject == issuer, lsfAccepted set automatically)
//   3. delete by subject before accept
//   4. delete by issuer before accept
//   5. verify lsfAccepted flag set on Credential ledger object after accept

use crate::common::{generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::account_objects::{AccountObjectType, AccountObjects},
    results,
    transactions::{
        credential_accept::CredentialAccept, credential_create::CredentialCreate,
        credential_delete::CredentialDelete,
    },
};

const CREDENTIAL_TYPE: &str = "4B5943"; // hex for "KYC"

// ── 1. Full lifecycle: create → accept → delete by issuer ────────────────────

#[tokio::test]
async fn test_credential_create_accept_delete() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;

        let mut create = CredentialCreate::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            subject.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
            None,
            None,
        );
        test_transaction(&mut create, &issuer).await;

        let mut accept = CredentialAccept::new(
            subject.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            issuer.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut accept, &subject).await;

        let mut delete = CredentialDelete::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Some(subject.classic_address.clone().into()),
            None,
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut delete, &issuer).await;
    })
    .await;
}

// ── 2. Self-issued credential: subject == issuer → lsfAccepted auto-set ──────

#[tokio::test]
async fn test_credential_create_self_issued() {
    with_blockchain_lock(|| async {
        let account = generate_funded_wallet().await;

        // When subject == issuer, rippled sets lsfAccepted automatically (no
        // CredentialAccept required). Confirm the create succeeds.
        let mut create = CredentialCreate::new(
            account.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            account.classic_address.clone().into(), // subject == issuer
            CREDENTIAL_TYPE.into(),
            None,
            None,
        );
        test_transaction(&mut create, &account).await;

        // Verify the Credential ledger object exists and lsfAccepted is set (0x00010000).
        let client = get_client().await;
        let ao_req = AccountObjects::new(
            None,
            account.classic_address.clone().into(),
            None,
            None,
            Some(AccountObjectType::Credential),
            None,
            None,
            None,
        );
        let ao_resp = client
            .request(ao_req.into())
            .await
            .expect("account_objects request failed");
        let ao_result: results::account_objects::AccountObjects<'_> =
            ao_resp.try_into().expect("parse account_objects");

        assert!(
            !ao_result.account_objects.is_empty(),
            "expected at least one Credential object for self-issued account"
        );
        let cred_obj = &ao_result.account_objects[0];
        let flags = cred_obj["Flags"].as_u64().unwrap_or(0);
        const LSF_ACCEPTED: u64 = 0x00010000;
        assert!(
            flags & LSF_ACCEPTED != 0,
            "lsfAccepted (0x00010000) should be set on self-issued credential, got Flags={flags:#010x}"
        );
    })
    .await;
}

// ── 3. Delete by subject before accept ──────────────────────────────────────

#[tokio::test]
async fn test_credential_delete_by_subject_before_accept() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;

        let mut create = CredentialCreate::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            subject.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
            None,
            None,
        );
        test_transaction(&mut create, &issuer).await;

        // Subject deletes the credential before accepting it.
        let mut delete = CredentialDelete::new(
            subject.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            None,                                           // subject omitted → defaults to Account
            Some(issuer.classic_address.clone().into()),    // issuer explicit
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut delete, &subject).await;
    })
    .await;
}

// ── 4. Delete by issuer before accept ───────────────────────────────────────

#[tokio::test]
async fn test_credential_delete_by_issuer_before_accept() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;

        let mut create = CredentialCreate::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            subject.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
            None,
            None,
        );
        test_transaction(&mut create, &issuer).await;

        // Issuer deletes the credential before subject accepts.
        let mut delete = CredentialDelete::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Some(subject.classic_address.clone().into()), // subject explicit
            None,                                         // issuer omitted → defaults to Account
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut delete, &issuer).await;
    })
    .await;
}

// ── 5. Verify lsfAccepted set after CredentialAccept ────────────────────────

#[tokio::test]
async fn test_credential_lsf_accepted_set_after_accept() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;

        let mut create = CredentialCreate::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            subject.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
            None,
            None,
        );
        test_transaction(&mut create, &issuer).await;

        // Before accept: lsfAccepted should NOT be set.
        let client = get_client().await;
        let ao_req = AccountObjects::new(
            None,
            subject.classic_address.clone().into(),
            None,
            None,
            Some(AccountObjectType::Credential),
            None,
            None,
            None,
        );
        let ao_resp = client
            .request(ao_req.into())
            .await
            .expect("account_objects request failed");
        let ao_before: results::account_objects::AccountObjects<'_> =
            ao_resp.try_into().expect("parse account_objects before");

        assert!(
            !ao_before.account_objects.is_empty(),
            "credential object should exist after create"
        );
        let flags_before = ao_before.account_objects[0]["Flags"].as_u64().unwrap_or(0);
        const LSF_ACCEPTED: u64 = 0x00010000;
        assert_eq!(
            flags_before & LSF_ACCEPTED,
            0,
            "lsfAccepted should NOT be set before accept, got Flags={flags_before:#010x}"
        );

        // Accept the credential.
        let mut accept = CredentialAccept::new(
            subject.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            issuer.classic_address.clone().into(),
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut accept, &subject).await;

        // After accept: lsfAccepted must be set.
        let ao_req2 = AccountObjects::new(
            None,
            subject.classic_address.clone().into(),
            None,
            None,
            Some(AccountObjectType::Credential),
            None,
            None,
            None,
        );
        let ao_resp2 = client
            .request(ao_req2.into())
            .await
            .expect("account_objects (after accept) failed");
        let ao_after: results::account_objects::AccountObjects<'_> =
            ao_resp2.try_into().expect("parse account_objects after");

        assert!(
            !ao_after.account_objects.is_empty(),
            "credential object should still exist after accept"
        );
        let flags_after = ao_after.account_objects[0]["Flags"].as_u64().unwrap_or(0);
        assert!(
            flags_after & LSF_ACCEPTED != 0,
            "lsfAccepted (0x00010000) should be set after accept, got Flags={flags_after:#010x}"
        );

        // Cleanup: delete the accepted credential (issuer can still delete).
        let mut delete = CredentialDelete::new(
            issuer.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Some(subject.classic_address.clone().into()),
            None,
            CREDENTIAL_TYPE.into(),
        );
        test_transaction(&mut delete, &issuer).await;
    })
    .await;
}

// ── Helper: provision a credential (create + accept) for use in other tests ─

#[cfg(feature = "std")]
pub async fn provision_credential(
    issuer: &xrpl::wallet::Wallet,
    subject: &xrpl::wallet::Wallet,
    credential_type: &str,
) {
    let mut create = CredentialCreate::new(
        issuer.classic_address.clone().into(),
        None, None, None, None, None, None, None, None,
        subject.classic_address.clone().into(),
        credential_type.into(),
        None,
        None,
    );
    test_transaction(&mut create, issuer).await;

    let mut accept = CredentialAccept::new(
        subject.classic_address.clone().into(),
        None, None, None, None, None, None, None, None,
        issuer.classic_address.clone().into(),
        credential_type.into(),
    );
    test_transaction(&mut accept, subject).await;
    ledger_accept().await;
}
