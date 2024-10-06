use xrpl::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
use xrpl::models::requests::account_info::AccountInfo;

fn main() {
    // connect to a XRP Ledger node
    let client = JsonRpcClient::connect("https://xrplcluster.com/".parse().unwrap());
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
    let response = client.request(account_info.into()).unwrap();
    println!("account info: {:?}", response);
}
