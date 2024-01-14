#![allow(dead_code)] // Remove eventually
mod common;

mod integration;

use anyhow::Result;

#[tokio::test]
async fn test_asynch_clients() -> Result<()> {
    #[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
    return integration::clients::test_websocket_tungstenite_echo().await;
    #[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
    return integration::clients::test_embedded_websocket_echo().await;
}
