// Scenarios:
//   - base: get an entry index from ledger_data, then query ledger_entry with that index
//   - credential: provision a Credential object, fetch it via ledger_entry credential selector

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock, CREDENTIAL_TYPE_KYC};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::{
        ledger_data::LedgerData as LedgerDataRequest,
        ledger_entry::{Credential as CredentialSelector, LedgerEntry},
        LedgerIndex,
    },
    results::ledger_data::LedgerData as LedgerDataResult,
    transactions::{
        credential_accept::CredentialAccept, credential_create::CredentialCreate, CommonFields,
        TransactionType,
    },
};

#[tokio::test]
async fn test_ledger_entry_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        // First, get a valid entry index from ledger_data
        let data_request = LedgerDataRequest::new(
            None,                                       // id
            None,                                       // binary
            None,                                       // ledger_hash
            Some(LedgerIndex::Str("validated".into())), // ledger_index
            Some(1),                                    // limit
            None,                                       // marker
        );

        let data_response = client
            .request(data_request.into())
            .await
            .expect("ledger_data request failed");

        let data_result: LedgerDataResult = data_response
            .try_into()
            .expect("failed to parse ledger_data result");

        let entry_index = data_result.state[0].index.clone();

        // Now query ledger_entry with that index
        let entry_request = LedgerEntry {
            index: Some(entry_index.clone()),
            ..Default::default()
        };

        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("failed ledger_entry request");

        let entry_result: xrpl::models::results::ledger_entry::LedgerEntry = entry_response
            .try_into()
            .expect("failed to parse ledger_entry result");

        // Verify the returned index matches what we requested
        assert_eq!(entry_result.index.as_ref(), entry_index.as_ref());
        // Verify the node is present (non-binary mode)
        assert!(entry_result.node.is_some());
    })
    .await;
}

// ── credential: provision a Credential then fetch it via ledger_entry selector ─

const CREDENTIAL_TYPE: &str = CREDENTIAL_TYPE_KYC;
const LSF_ACCEPTED: u64 = 0x00010000;

#[tokio::test]
async fn test_ledger_entry_credential() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let issuer = generate_funded_wallet().await;
        let subject = generate_funded_wallet().await;

        // Step 1: create credential (issuer → subject).
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

        // Step 2: subject accepts — sets lsfAccepted flag.
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

        // Step 3: fetch via ledger_entry credential selector.
        let entry_request = LedgerEntry {
            credential: Some(CredentialSelector {
                subject: subject.classic_address.clone().into(),
                issuer: issuer.classic_address.clone().into(),
                credential_type: CREDENTIAL_TYPE.into(),
            }),
            ..Default::default()
        };

        let entry_response = client
            .request(entry_request.into())
            .await
            .expect("ledger_entry credential request failed");

        let entry_result: xrpl::models::results::ledger_entry::LedgerEntry = entry_response
            .try_into()
            .expect("failed to parse ledger_entry result");

        assert!(entry_result.node.is_some(), "node should be present");

        // Verify lsfAccepted is set.
        let node = entry_result.node.unwrap();
        let flags = node["Flags"].as_u64().expect("Flags field missing");
        assert!(
            flags & LSF_ACCEPTED != 0,
            "lsfAccepted should be set after CredentialAccept, got Flags={flags:#010x}"
        );
    })
    .await;
}
