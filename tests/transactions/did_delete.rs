// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/didSet.test.ts
// (DIDDelete is tested alongside DIDSet in other client libraries)
//
// Scenarios:
//   - base: create a DID then delete it, verify the DID is removed
//   - delete_nonexistent: attempt to delete a DID that doesn't exist, expect tecNO_ENTRY

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::did_delete::DIDDelete;
use xrpl::models::transactions::did_set::DIDSet;

const DATA_HEX: &str = "617474657374";
const DID_DOCUMENT_HEX: &str = "646F63";
const URI_HEX: &str = "6469645F6578616D706C65";

#[tokio::test]
async fn test_did_delete_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let client = get_client().await;

        // Step 1: Create a DID
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

        // Verify DID exists
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
            "DID should exist after DIDSet"
        );

        // Step 2: Delete the DID
        let mut delete_tx = DIDDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        test_transaction(&mut delete_tx, &wallet).await;

        // Step 3: Verify DID is gone
        let ao_response2 = client
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

        let objects_result2: results::account_objects::AccountObjects<'_> = ao_response2
            .try_into()
            .expect("failed to parse account_objects result");
        assert!(
            objects_result2.account_objects.is_empty(),
            "DID should be removed after DIDDelete"
        );
    })
    .await;
}

#[tokio::test]
async fn test_did_delete_nonexistent() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let client = get_client().await;

        // Attempt to delete a DID that doesn't exist
        let mut tx = DIDDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit should not error at network level");

        assert!(
            result.engine_result.contains("tecNO_ENTRY"),
            "Expected tecNO_ENTRY when deleting non-existent DID, got: {}",
            result.engine_result
        );

        ledger_accept().await;
    })
    .await;
}
