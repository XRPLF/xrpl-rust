use xrpl::asynch::client::{AsyncWebsocketClientTungstenite, TungsteniteMessage};
use xrpl::models::requests::AccountInfo;

#[tokio::main]
async fn main() {
    let websocket =
        AsyncWebsocketClientTungstenite::open("wss://xrplcluster.com/".parse().unwrap())
            .await
            .unwrap();
    assert!(websocket.is_open());

    let account_info = AccountInfo::new(
        "rJumr5e1HwiuV543H7bqixhtFreChWTaHH",
        None,
        None,
        None,
        None,
        None,
        None,
    );

    websocket.send(&account_info).await.unwrap();

    while let Ok(Some(TungsteniteMessage::Text(response))) = websocket.try_next().await {
        let account_info_echo: AccountInfo = serde_json::from_str(response.as_str()).unwrap();
        println!("account_info_echo: {:?}", account_info_echo);

        websocket.close().await.unwrap();
        break;
    }
}
