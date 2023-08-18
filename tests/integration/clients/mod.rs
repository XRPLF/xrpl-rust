use xrpl::models::requests::AccountInfo;

#[tokio::test]
#[cfg(feature = "tungstenite")]
async fn test_websocket_tungstenite_echo() {
    use super::*;
    use super::super::common::connect_to_wss_tungstinite_echo;
    use xrpl::asynch::clients::async_websocket_client::TungsteniteMessage;
    use futures::{SinkExt, TryStreamExt};

    let mut websocket = connect_to_wss_tungstinite_echo().await;
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
        assert_eq!(account_info, account_info_echo);

        break;
    }
}

#[tokio::test]
#[cfg(feature = "embedded-websocket")]
async fn test_embedded_websocket_echo() {
    use super::*;
    use super::super::common::{codec::Codec, connect_to_ws_embedded_websocket_tokio_echo};
    use xrpl::asynch::clients::async_websocket_client::EmbeddedWebsocketReadMessageType;
    use tokio_util::codec::Framed;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let mut framed = Framed::new(tcp_stream, Codec::new());
    let mut buffer = [0u8; 4096];
    let mut websocket = connect_to_ws_embedded_websocket_tokio_echo(&mut framed, &mut buffer).await;
    let account_info = AccountInfo::new(
        "rJumr5e1HwiuV543H7bqixhtFreChWTaHH",
        None,
        None,
        None,
        None,
        None,
        None,
    );
    websocket
        .send(&mut framed, &mut buffer, false, &account_info)
        .await
        .unwrap();

    let mut ping_counter = 0;
    loop {
        let message = websocket
            .try_next(&mut framed, &mut buffer)
            .await
            .unwrap()
            .unwrap();
        match message {
            EmbeddedWebsocketReadMessageType::Ping(_) => {
                ping_counter += 1;
                if ping_counter > 1 {
                    panic!("Expected only one ping");
                }
            }
            EmbeddedWebsocketReadMessageType::Text(text) => {
                assert_eq!(
                    serde_json::from_str::<AccountInfo>(text).unwrap(),
                    account_info
                );
                break;
            }
            EmbeddedWebsocketReadMessageType::Binary(_) => {
                panic!("Expected text message found binary")
            }
            EmbeddedWebsocketReadMessageType::Pong(_) => {
                panic!("Expected text message found pong")
            }
            EmbeddedWebsocketReadMessageType::Close(_) => {
                panic!("Expected text message found close")
            }
        }
    }
}
