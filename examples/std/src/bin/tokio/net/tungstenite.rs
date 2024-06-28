use serde_json::Value;
use tokio_tungstenite::connect_async;
use xrpl::asynch::clients::{
    AsyncWebsocketClient, SingleExecutorMutex, WebsocketOpen, XRPLWebsocketIO,
};
use xrpl::models::requests::{StreamParameter, Subscribe};

#[tokio::main]
async fn main() {
    let stream = connect_async("wss://xrplcluster.com/").await.unwrap().0;
    let mut websocket: AsyncWebsocketClient<_, SingleExecutorMutex, WebsocketOpen> =
        AsyncWebsocketClient::open(stream).await.unwrap();
    let subscribe = Subscribe::new(
        None,
        None,
        None,
        None,
        Some(vec![StreamParameter::Ledger]),
        None,
        None,
        None,
    );
    websocket.xrpl_send(subscribe).await.unwrap();
    while let Ok(Some(account_info_echo)) = websocket.xrpl_receive::<Value, Subscribe>().await {
        println!("subscription message: {:?}", account_info_echo);
        break;
    }
}
