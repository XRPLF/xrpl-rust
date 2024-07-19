use tokio_tungstenite::connect_async;
use xrpl::asynch::clients::{
    AsyncWebsocketClient, SingleExecutorMutex, WebsocketOpen, XRPLWebsocketIO,
};
use xrpl::models::requests::{StreamParameter, Subscribe};

#[tokio::main]
async fn main() {
    let mut websocket: AsyncWebsocketClient<SingleExecutorMutex, _> =
        AsyncWebsocketClient::open("wss://xrplcluster.com/".parse().unwrap())
            .await
            .unwrap();
    let subscribe = Subscribe::new(
        Some("my_id".into()),
        None,
        None,
        None,
        Some(vec![StreamParameter::Ledger]),
        None,
        None,
        None,
    );
    websocket.xrpl_send(subscribe.into()).await.unwrap();
    loop {
        let account_info_echo = websocket.xrpl_receive().await.unwrap().unwrap();
        println!("subscription message: {:?}", account_info_echo);
        // break;
    }
}
