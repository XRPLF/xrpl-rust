#![allow(dead_code)] // Remove eventually
mod common;

mod integration;

use anyhow::Result;

#[cfg(any(feature = "websocket-std", all(feature = "websocket", feature = "std")))]
#[tokio::test]
async fn test_asynch_clients() -> Result<()> {
    #[cfg(all(feature = "websocket-std", not(feature = "websocket")))]
    return integration::clients::test_websocket_tungstenite_test_net().await;
    #[cfg(all(feature = "websocket", feature = "std", not(feature = "websocket-std")))]
    return integration::clients::test_embedded_websocket_echo().await;
    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(any(feature = "websocket-std", feature = "websocket", feature = "std"))]
#[tokio::test]
async fn test_asynch_clients_request() -> Result<()> {
    #[cfg(all(feature = "websocket-std", feature = "std", not(feature = "websocket")))]
    return integration::clients::test_websocket_tungstenite_request().await;
    #[cfg(all(feature = "websocket", feature = "std", not(feature = "websocket-std")))]
    return integration::clients::test_embedded_websocket_request().await;
    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(all(feature = "json-rpc-std", not(feature = "json-rpc")))]
#[tokio::test]
async fn test_asynch_clients_json_rpc() -> Result<()> {
    #[cfg(all(feature = "json-rpc-std", not(feature = "json-rpc")))]
    return integration::clients::test_json_rpc_std().await;
    #[allow(unreachable_code)]
    Ok(())
}
