use anyhow::Result;

#[cfg(all(feature = "websocket-std", not(feature = "websocket")))]
pub async fn test_websocket_tungstenite_test_net() -> Result<()> {
    use crate::common::connect_to_wss_tungstinite_test_net;
    use xrpl::{asynch::clients::XRPLWebsocketIO, models::requests::Fee};

    let mut websocket = connect_to_wss_tungstinite_test_net().await?;
    let fee = Fee::new(None);

    websocket.xrpl_send(fee.into()).await.unwrap();
    let message = websocket.xrpl_receive().await.unwrap();
    assert!(message.unwrap().result.is_some());
    Ok(())
}

#[cfg(all(feature = "websocket-std", not(feature = "websocket")))]
pub async fn test_websocket_tungstenite_request() -> Result<()> {
    use crate::common::connect_to_wss_tungstinite_test_net;
    use xrpl::{asynch::clients::AsyncClient, models::requests::Fee};

    let websocket = connect_to_wss_tungstinite_test_net().await?;
    let fee = Fee::new(None);

    let message = websocket.request(fee.into()).await.unwrap();
    assert!(message.result.is_some());
    Ok(())
}

#[cfg(all(feature = "websocket", feature = "std", not(feature = "websocket-std")))]
pub async fn test_embedded_websocket_echo() -> Result<()> {
    use crate::common::connect_to_ws_embedded_websocket_tokio_echo;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::codec::Codec;
    use xrpl::asynch::clients::XRPLWebsocketIO;
    use xrpl::models::requests::Fee;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let framed = Framed::new(tcp_stream, Codec);
    let mut websocket = connect_to_ws_embedded_websocket_tokio_echo(framed).await?;
    let fee = Fee::new(None);
    websocket.xrpl_send(fee.into()).await?;
    let _ = websocket.xrpl_receive().await.unwrap();
    Ok(())
}

#[cfg(all(feature = "websocket", feature = "std", not(feature = "websocket-std")))]
pub async fn test_embedded_websocket_request() -> Result<()> {
    use crate::common::connect_to_ws_embedded_websocket_tokio_echo;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::codec::Codec;
    use xrpl::asynch::clients::AsyncClient;
    use xrpl::models::requests::Fee;

    let tcp_stream = tokio::net::TcpStream::connect("ws.vi-server.org:80")
        .await
        .unwrap();
    let framed = Framed::new(tcp_stream, Codec);
    let websocket = connect_to_ws_embedded_websocket_tokio_echo(framed).await?;
    let fee = Fee::new(None);
    let _res = websocket.request(fee.into()).await?;
    Ok(())
}

#[cfg(all(feature = "json-rpc-std", not(feature = "json-rpc")))]
pub async fn test_json_rpc_std() -> Result<()> {
    use xrpl::{
        asynch::clients::{AsyncClient, AsyncJsonRpcClient, SingleExecutorMutex},
        models::requests::Fee,
    };
    let client: AsyncJsonRpcClient<SingleExecutorMutex> =
        AsyncJsonRpcClient::new("https://s1.ripple.com:51234/".parse().unwrap());
    let fee_result = client.request(Fee::new(None).into()).await.unwrap();
    assert!(fee_result.result.is_some());
    Ok(())
}
