// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/checkCash.test.ts
//
// Scenarios:
//   - base: create a Check for 500 drops, then cash it for the exact amount

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit};
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::check_cash::CheckCash;
use xrpl::models::transactions::check_create::CheckCreate;
use xrpl::models::{Amount, XRPAmount};

#[tokio::test]
async fn test_check_cash_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let amount = "500";

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
            destination.classic_address.clone().into(), // destination
            Amount::XRPAmount(XRPAmount::from(amount)), // send_max
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
        let ao_result: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("Failed to parse account_objects");

        assert_eq!(ao_result.account_objects.len(), 1, "Expected one check");

        let check_id = ao_result.account_objects[0]["index"]
            .as_str()
            .expect("Expected index field on check object")
            .to_string();

        // Step 3: cash the check (destination receives the funds)
        let mut cash_tx = CheckCash::new(
            destination.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            check_id.into(),                                  // check_id
            Some(Amount::XRPAmount(XRPAmount::from(amount))), // amount (exact)
            None,                                             // deliver_min
        );

        test_transaction(&mut cash_tx, &destination).await;

        // Confirm the check was consumed
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
            .expect("Failed to query account_objects after cash");
        let ao_result2: results::account_objects::AccountObjects<'_> = ao_response2
            .try_into()
            .expect("Failed to parse account_objects");

        assert_eq!(
            ao_result2.account_objects.len(),
            0,
            "Check should be gone after cashing"
        );
    })
    .await;
}
