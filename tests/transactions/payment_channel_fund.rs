// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/paymentChannelFund.test.ts
//
// Scenarios:
//   - base: create a channel, then add 100 more drops via PaymentChannelFund
//
// NOTE: xrpl.js computes the channel ID via hashPaymentChannel(account, dest, seq).
// xrpl-rust has no equivalent utility, so we read the channel ID from account_objects
// after the PaymentChannelCreate is validated.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::results;
use xrpl::models::transactions::payment_channel_create::PaymentChannelCreate;
use xrpl::models::transactions::payment_channel_fund::PaymentChannelFund;
use xrpl::models::XRPAmount;

#[tokio::test]
async fn test_payment_channel_fund_base() {
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

        submit_and_wait(&mut create_tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit PaymentChannelCreate");

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
        let ao_result: results::account_objects::AccountObjects<'_> =
            ao_response.try_into().expect("Failed to parse account_objects");

        assert_eq!(ao_result.account_objects.len(), 1, "Expected one channel");

        let channel_id = ao_result.account_objects[0]["index"]
            .as_str()
            .expect("Expected index field on channel object")
            .to_string();

        // Step 3: fund the channel with an additional 100 drops
        let mut fund_tx = PaymentChannelFund::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("100"), // amount: 100 more drops
            channel_id.into(),      // channel
            None,                   // expiration
        );

        let result = submit_and_wait(&mut fund_tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit PaymentChannelFund");

        assert_eq!(
            result
                .get_transaction_metadata()
                .expect("Expected metadata")
                .transaction_result,
            "tesSUCCESS"
        );

        ledger_accept().await;
    })
    .await;
}
