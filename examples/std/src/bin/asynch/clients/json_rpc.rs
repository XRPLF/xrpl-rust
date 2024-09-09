use xrpl::asynch::clients::{AsyncJsonRpcClient, XRPLAsyncClient};
use xrpl::models::requests::account_info::AccountInfo;

#[tokio::main]
async fn main() {
    // connect to a XRP Ledger node
    let client = AsyncJsonRpcClient::connect("https://xrplcluster.com/".parse().unwrap());
    // request account info
    let account_info = AccountInfo::new(
        None,
        "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59".into(),
        None,
        None,
        None,
        None,
        None,
    );
    let response = client.request(account_info.into()).await.unwrap();
    println!("account info: {:?}", response);
}
