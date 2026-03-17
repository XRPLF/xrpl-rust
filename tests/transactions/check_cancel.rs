// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/checkCancel.test.ts
//
// Scenarios:
//   - base: create a Check for 50 drops, then cancel it (creator cancels their own check)

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit};
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::check_cancel::CheckCancel;
use xrpl::models::transactions::check_create::CheckCreate;
use xrpl::models::{Amount, XRPAmount};

#[tokio::test]
async fn test_check_cancel_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        // Step 1: create the check
        let mut create_tx = CheckCreate::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            destination.classic_address.clone().into(),       // destination
            Amount::XRPAmount(XRPAmount::from("50")), // send_max: 50 drops
            None,
            None,
            None,
        );

        sign_and_submit(&mut create_tx, client, &wallet, true, true)
            .await
            .expect("Failed to submit CheckCreate");

        ledger_accept().await;

        // Step 2: get the check ID from account_objects
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::Check),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("Failed to query account_objects");
        let ao_result: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("Failed to parse account_objects");

        assert_eq!(ao_result.account_objects.len(), 1, "Expected one check");

        let check_id = ao_result.account_objects[0]["index"]
            .as_str()
            .expect("Expected index field on check object")
            .to_string();

        // Step 3: cancel the check (creator cancels their own check)
        let mut cancel_tx = CheckCancel::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            check_id.into(), // check_id
        );

        test_transaction(&mut cancel_tx, &wallet).await;

        // Confirm the check no longer exists
        let ao_response2 = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::Check),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("Failed to query account_objects after cancel");
        let ao_result2: results::account_objects::AccountObjects<'_> =
            ao_response2.try_into().expect("Failed to parse account_objects");

        assert_eq!(
            ao_result2.account_objects.len(),
            0,
            "Check should be gone after cancelling"
        );
    })
    .await;
}
