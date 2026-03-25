// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/checkCreate.test.ts
//
// Scenarios:
//   - base: create a Check for 50 drops and verify one check object exists on the ledger

use crate::common::{generate_funded_wallet, get_client, test_transaction, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::check_create::CheckCreate;
use xrpl::models::{Amount, XRPAmount};

#[tokio::test]
async fn test_check_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        let mut tx = CheckCreate::new(
            wallet.classic_address.clone().into(),
            None,                                       // account_txn_id
            None,                                       // fee
            None,                                       // last_ledger_sequence
            None,                                       // memos
            None,                                       // sequence
            None,                                       // signers
            None,                                       // source_tag
            None,                                       // ticket_sequence
            destination.classic_address.clone().into(), // destination
            Amount::XRPAmount(XRPAmount::from("50")),   // send_max: 50 drops
            None,                                       // destination_tag
            None,                                       // expiration
            None,                                       // invoice_id
        );

        test_transaction(&mut tx, &wallet).await;

        // Verify the check ledger object was created
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

        assert_eq!(
            ao_result.account_objects.len(),
            1,
            "Should be exactly one check on the ledger"
        );
    })
    .await;
}
