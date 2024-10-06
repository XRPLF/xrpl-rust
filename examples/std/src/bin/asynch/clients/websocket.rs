use xrpl::asynch::clients::{
    AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen, XRPLAsyncWebsocketIO,
};
use xrpl::models::requests::subscribe::{StreamParameter, Subscribe};

#[tokio::main]
async fn main() {
    // open a websocket connection to a XRP Ledger node
    let mut websocket: AsyncWebSocketClient<SingleExecutorMutex, _> =
        AsyncWebSocketClient::open("wss://xrplcluster.com/".parse().unwrap())
            .await
            .unwrap();
    // subscribe to the ledger stream
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
    websocket.xrpl_send(subscribe.into()).await.unwrap();
    // listen for messages
    loop {
        let account_info_echo = websocket.xrpl_receive().await.unwrap().unwrap();
        println!("subscription message: {:?}", account_info_echo);
    }
}
