use anyhow::anyhow;
use anyhow::Result;

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use std::io;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use tokio::net::TcpStream;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use tokio_util::codec::Framed;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use xrpl::asynch::clients::codec::Codec;
use xrpl::asynch::clients::AsyncWebsocketClient;
use xrpl::asynch::clients::{SingleExecutorMutex, WebsocketOpen};

mod constants;
pub use constants::*;

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub async fn connect_to_wss_tungstinite_test_net(
) -> Result<AsyncWebsocketClient<SingleExecutorMutex, WebsocketOpen>> {
    match XRPL_WSS_TEST_NET.parse() {
        Ok(url) => match AsyncWebsocketClient::open(url).await {
            Ok(websocket) => {
                // assert!(websocket.is_open());
                Ok(websocket)
            }
            Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
        },
        Err(err) => Err(anyhow!("Error parsing url: {:?}", err)),
    }
}

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
pub async fn connect_to_ws_embedded_websocket_tokio_echo(
    stream: Framed<TcpStream, Codec>,
) -> Result<
    AsyncWebsocketClient<
        4096,
        Framed<TcpStream, Codec>,
        Vec<u8>,
        io::Error,
        rand_core::OsRng,
        SingleExecutorMutex,
        WebsocketOpen,
    >,
> {
    let rng = rand_core::OsRng;
    let url = ECHO_WS_SERVER.parse().unwrap();
    match AsyncWebsocketClient::open(rng, stream, url).await {
        Ok(websocket) => {
            // assert!(websocket.is_open());
            Ok(websocket)
        }
        Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
    }
}
