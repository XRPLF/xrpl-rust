use anyhow::Result;

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub async fn test_websocket_tungstenite_test_net() -> Result<()> {
    use crate::common::connect_to_wss_tungstinite_test_net;
    use xrpl::{
        asynch::clients::XRPLWebsocketIO, models::requests::Fee, models::results::FeeResult,
    };

    let mut websocket = connect_to_wss_tungstinite_test_net().await?;
    let fee = Fee::new(None);

    websocket.xrpl_send(fee).await.unwrap();
    let message = websocket
        .xrpl_receive::<FeeResult<'_>, Fee<'_>>()
        .await
        .unwrap();
    assert!(message.unwrap().result.is_some());
    Ok(())
}

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub async fn test_websocket_tungstenite_request() -> Result<()> {
    use crate::common::connect_to_wss_tungstinite_test_net;
    use xrpl::{asynch::clients::AsyncClient, models::requests::Fee, models::results::FeeResult};

    let websocket = connect_to_wss_tungstinite_test_net().await?;
    let fee = Fee::new(None);

    let message = websocket.request::<FeeResult<'_>, _>(fee).await.unwrap();
    assert!(message.result.is_some());
    Ok(())
}

#[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
pub async fn test_embedded_websocket_echo() -> Result<()> {
    use crate::common::connect_to_ws_embedded_websocket_tokio_echo;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::codec::Codec;
    use xrpl::asynch::clients::XRPLWebsocketIO;
    use xrpl::models::requests::Fee;
    use xrpl::models::results::FeeResult;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let framed = Framed::new(tcp_stream, Codec);
    let mut websocket = connect_to_ws_embedded_websocket_tokio_echo(framed).await?;
    let fee = Fee::new(None);
    websocket.xrpl_send(fee).await?;
    let _ = websocket
        .xrpl_receive::<FeeResult<'_>, Fee<'_>>()
        .await
        .unwrap();
    Ok(())
}

#[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
pub async fn test_embedded_websocket_request() -> Result<()> {
    use crate::common::connect_to_ws_embedded_websocket_tokio_echo;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::codec::Codec;
    use xrpl::asynch::clients::AsyncClient;
    use xrpl::models::requests::Fee;
    use xrpl::models::results::FeeResult;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let framed = Framed::new(tcp_stream, Codec);
    let websocket = connect_to_ws_embedded_websocket_tokio_echo(framed).await?;
    let fee = Fee::new(None);
    let _res = websocket.request::<FeeResult, _>(fee).await?;
    Ok(())
}

#[cfg(feature = "json-rpc-std")]
pub async fn test_json_rpc_std() -> Result<()> {
    use xrpl::{
        asynch::clients::{AsyncClient, AsyncJsonRpcClient, SingleExecutorMutex},
        models::requests::Fee,
        models::results::FeeResult,
    };
    let client: AsyncJsonRpcClient<SingleExecutorMutex> =
        AsyncJsonRpcClient::new("https://s1.ripple.com:51234/".parse().unwrap());
    let fee_result = client
        .request::<FeeResult, _>(Fee::new(None))
        .await
        .unwrap();
    assert!(fee_result.result.is_some());
    Ok(())
}
