#![allow(dead_code)] // Remove eventually
mod common;

mod integration;

use anyhow::Result;

#[cfg(any(feature = "tungstenite", all(feature = "embedded-ws", feature = "std")))]
#[tokio::test]
async fn test_asynch_clients() -> Result<()> {
    #[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
    return integration::clients::test_websocket_tungstenite_test_net().await;
    #[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
    return integration::clients::test_embedded_websocket_echo().await;
    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(any(feature = "tungstenite", feature = "embedded-ws", feature = "std"))]
#[tokio::test]
async fn test_asynch_clients_request() -> Result<()> {
    #[cfg(all(feature = "tungstenite", feature = "std", not(feature = "embedded-ws")))]
    return integration::clients::test_websocket_tungstenite_request().await;
    #[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
    return integration::clients::test_embedded_websocket_request().await;
    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(feature = "json-rpc")]
#[tokio::test]
async fn test_asynch_clients_json_rpc() -> Result<()> {
    #[cfg(feature = "json-rpc")]
    return integration::clients::test_json_rpc().await;
    #[allow(unreachable_code)]
    Ok(())
}
