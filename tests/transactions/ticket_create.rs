// Scenarios:
//   - base: create 2 tickets and verify both ticket objects appear in account_objects

use crate::common::{generate_funded_wallet, get_client, test_transaction, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::ticket_create::TicketCreate;

#[tokio::test]
async fn test_ticket_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = TicketCreate::new(
            wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            2,    // ticket_count: create 2 tickets
        );

        test_transaction(&mut tx, &wallet).await;

        // Verify both ticket objects were created on the ledger.
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::Ticket),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("Failed to query account_objects for tickets");

        let ao_result: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("Failed to parse account_objects");

        assert_eq!(
            ao_result.account_objects.len(),
            2,
            "Expected 2 ticket objects on the ledger but found {}",
            ao_result.account_objects.len()
        );
    })
    .await;
}
