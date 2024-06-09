use anyhow::Result;

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub async fn test_websocket_tungstenite_echo() -> Result<()> {
    use super::common::connect_to_wss_tungstinite_echo;
    use xrpl::{asynch::clients::XRPLWebsocketIO, models::requests::AccountInfo};

    let mut websocket = connect_to_wss_tungstinite_echo().await?;
    let account_info = AccountInfo::new(
        None,
        "rJumr5e1HwiuV543H7bqixhtFreChWTaHH".into(),
        None,
        None,
        None,
        None,
        None,
    );

    websocket.xrpl_send(account_info).await.unwrap();
    let _ = websocket
        .xrpl_receive::<AccountInfo<'_>, AccountInfo<'_>>()
        .await
        .unwrap();
    Ok(())
}

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
pub async fn test_embedded_websocket_echo() -> Result<()> {
    use super::common::connect_to_ws_embedded_websocket_tokio_echo;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::codec::Codec;
    use xrpl::asynch::clients::XRPLWebsocketIO;
    use xrpl::models::requests::AccountInfo;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let mut framed = Framed::new(tcp_stream, Codec);
    let mut websocket = connect_to_ws_embedded_websocket_tokio_echo(framed).await?;
    let account_info = AccountInfo::new(
        None,
        "rJumr5e1HwiuV543H7bqixhtFreChWTaHH".into(),
        None,
        None,
        None,
        None,
        None,
    );
    websocket.xrpl_send(account_info).await?;
    let _ = websocket
        .xrpl_receive::<AccountInfo<'_>, AccountInfo<'_>>()
        .await?;
    Ok(())
}
