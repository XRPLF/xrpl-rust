use xrpl::asynch::clients::AsyncJsonRpcClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::asynch::wallet::generate_faucet_wallet;
use xrpl::models::transactions::account_set::AccountSet;

#[tokio::main]
async fn main() {
    let client = AsyncJsonRpcClient::connect("https://testnet.xrpl-labs.com/".parse().unwrap());
    // Create a new wallet we can use to sign the transaction
    let wallet = generate_faucet_wallet(&client, None, None, None, None)
        .await
        .unwrap();
    // Define the transaction we want to sign
    let mut account_set = AccountSet::new(
        wallet.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("6578616d706c652e636f6d".into()), // example.com
        None,
        None,
        None,
        None,
        None,
        None,
    );
    println!("AccountSet transaction before signing: {:?}", account_set);
    // Sign and submit the transaction
    sign_and_submit(&mut account_set, &client, &wallet, true, true)
        .await
        .unwrap();
    println!("AccountSet transaction after signing: {:?}", account_set);
}
