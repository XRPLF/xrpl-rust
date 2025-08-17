use xrpl::asynch::clients::AsyncJsonRpcClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::asynch::wallet::generate_faucet_wallet;
use xrpl::models::transactions::{AccountSet, CommonFields, TransactionType};

#[tokio::main]
async fn main() {
    let client = AsyncJsonRpcClient::connect("https://testnet.xrpl-labs.com/".parse().unwrap());
    // Create a new wallet we can use to sign the transaction
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .unwrap();

    // Define the transaction using the builder pattern
    let mut account_set = AccountSet {
        common_fields: CommonFields {
            account: wallet.classic_address.clone().into(),
            transaction_type: TransactionType::AccountSet,
            ..Default::default()
        },
        ..Default::default()
    }
    .with_domain("6578616d706c652e636f6d".into()) // example.com
    .with_fee("12".into())
    .with_sequence(1); // This will be auto-filled if needed

    println!("AccountSet transaction before signing: {:?}", account_set);

    // Sign and submit the transaction
    sign_and_submit(&mut account_set, &client, &wallet, true, true)
        .await
        .unwrap();

    println!("AccountSet transaction after signing: {:?}", account_set);
}
