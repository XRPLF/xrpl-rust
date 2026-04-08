// Scenarios:
//   - base: create a channel, then submit a claim for 100 drops (channel source claims to destination)
//
// NOTE: PaymentChannelClaim has `flags` at parameter position 4 (after `fee`), the same
// anomaly as NFTokenMint and PaymentChannelClaim. Pass None for no flags.
//
// NOTE: `amount` in PaymentChannelClaim is `Option<Cow<'a, str>>` (raw drop string),
// not XRPAmount. Pass `Some("100".into())` for 100 drops.
//
// NOTE: We read the channel ID from account_objects since xrpl-rust has no
// hashPaymentChannel utility.

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::{clients::XRPLAsyncClient, transaction::sign_and_submit};
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::payment_channel_claim::PaymentChannelClaim;
use xrpl::models::transactions::payment_channel_create::PaymentChannelCreate;
use xrpl::models::XRPAmount;

#[tokio::test]
async fn test_payment_channel_claim_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;
        let destination = generate_funded_wallet().await;

        // Step 1: create the payment channel
        let mut create_tx = PaymentChannelCreate::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("100"),                     // amount: 100 drops
            destination.classic_address.clone().into(), // destination
            wallet.public_key.clone().into(),           // public_key
            86400,                                      // settle_delay
            None,
            None,
        );

        sign_and_submit(&mut create_tx, client, &wallet, true, true)
            .await
            .expect("Failed to submit PaymentChannelCreate");

        ledger_accept().await;

        // Step 2: get the channel ID from account_objects
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::PaymentChannel),
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

        assert_eq!(ao_result.account_objects.len(), 1, "Expected one channel");

        let channel_id = ao_result.account_objects[0]["index"]
            .as_str()
            .expect("Expected index field on channel object")
            .to_string();

        // Step 3: submit a claim for 100 drops (source claims the full channel balance)
        // flags is at position 4 in PaymentChannelClaim::new().
        let mut claim_tx = PaymentChannelClaim::new(
            wallet.classic_address.clone().into(),
            None,               // account_txn_id
            None,               // fee
            None,               // flags (position 4 — same anomaly as NFTokenMint)
            None,               // last_ledger_sequence
            None,               // memos
            None,               // sequence
            None,               // signers
            None,               // source_tag
            None,               // ticket_sequence
            channel_id.into(),  // channel
            Some("100".into()), // amount: 100 drops (Cow<str>, not XRPAmount)
            None,               // balance
            None,               // public_key
            None,               // signature
        );

        test_transaction(&mut claim_tx, &wallet).await;
    })
    .await;
}
