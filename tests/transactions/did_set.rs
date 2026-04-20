// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/didSet.test.ts
//
// Scenarios:
//   - base: create a DID with all three fields (data, did_document, uri)
//   - update: modify an existing DID by changing URI and clearing DIDDocument
//   - single_field: create a DID with only the data field
//   - empty_field_deletion: set a field to empty string to delete it from an existing DID

use crate::common::{generate_funded_wallet, get_client, test_transaction, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::did_set::DIDSet;

/// Hex-encoded values matching the xrpl.js integration tests:
/// "617474657374" = "attest"
/// "646F63" = "doc"
/// "6469645F6578616D706C65" = "did_example"
const DATA_HEX: &str = "617474657374";
const DID_DOCUMENT_HEX: &str = "646F63";
const URI_HEX: &str = "6469645F6578616D706C65";

#[tokio::test]
async fn test_did_set_all_fields() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut tx = DIDSet::new(
            wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            Some(DATA_HEX.into()),
            Some(DID_DOCUMENT_HEX.into()),
            Some(URI_HEX.into()),
        );

        test_transaction(&mut tx, &wallet).await;

        // Verify the DID was created by querying account_objects
        let client = get_client().await;
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::DID),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let objects_result: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("failed to parse account_objects result");

        assert_eq!(
            objects_result.account_objects.len(),
            1,
            "Should be exactly one DID on the ledger after DIDSet"
        );

        let did_obj = &objects_result.account_objects[0];
        assert_eq!(did_obj["Data"].as_str().unwrap(), DATA_HEX);
        assert_eq!(did_obj["DIDDocument"].as_str().unwrap(), DID_DOCUMENT_HEX);
        assert_eq!(did_obj["URI"].as_str().unwrap(), URI_HEX);
    })
    .await;
}

#[tokio::test]
async fn test_did_set_update() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        // Step 1: Create DID with all fields
        let mut create_tx = DIDSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(DATA_HEX.into()),
            Some(DID_DOCUMENT_HEX.into()),
            Some(URI_HEX.into()),
        );
        test_transaction(&mut create_tx, &wallet).await;

        // Step 2: Update DID — change URI, clear DIDDocument, leave Data unchanged
        let new_uri = "ABCD";
        let mut update_tx = DIDSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,                 // data: omit to leave unchanged
            Some("".into()),      // did_document: empty string to delete
            Some(new_uri.into()), // uri: update value
        );
        test_transaction(&mut update_tx, &wallet).await;

        // Step 3: Verify the update
        let client = get_client().await;
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::DID),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let objects_result: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("failed to parse account_objects result");

        assert_eq!(objects_result.account_objects.len(), 1);
        let did_obj = &objects_result.account_objects[0];
        // Data should be unchanged
        assert_eq!(did_obj["Data"].as_str().unwrap(), DATA_HEX);
        // DIDDocument should be removed (field absent from ledger object)
        assert!(
            did_obj.get("DIDDocument").is_none()
                || did_obj["DIDDocument"]
                    .as_str()
                    .map_or(true, |s| s.is_empty()),
            "DIDDocument should be cleared"
        );
        // URI should be updated
        assert_eq!(did_obj["URI"].as_str().unwrap(), new_uri);
    })
    .await;
}

#[tokio::test]
async fn test_did_set_empty_did_rejected() {
    // DIDSet with all empty string fields should fail client-side validation.
    // The sign_and_submit function also runs model validation, so this test
    // verifies that invalid DIDSet transactions are rejected before submission.
    let tx = DIDSet {
        common_fields: xrpl::models::transactions::CommonFields {
            account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            transaction_type: xrpl::models::transactions::TransactionType::DIDSet,
            ..Default::default()
        },
        data: Some("".into()),
        did_document: Some("".into()),
        uri: Some("".into()),
    };

    assert!(
        !xrpl::models::Model::is_valid(&tx),
        "DIDSet with all empty fields should fail client-side validation"
    );

    // Also verify that a DIDSet with all None fields is rejected
    let tx_none = DIDSet {
        common_fields: xrpl::models::transactions::CommonFields {
            account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
            transaction_type: xrpl::models::transactions::TransactionType::DIDSet,
            ..Default::default()
        },
        data: None,
        did_document: None,
        uri: None,
    };

    assert!(
        !xrpl::models::Model::is_valid(&tx_none),
        "DIDSet with no fields should fail client-side validation"
    );
}
