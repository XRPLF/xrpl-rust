pub mod codec;

use anyhow::anyhow;
use anyhow::Result;

#[cfg(feature = "embedded-websocket")]
use tokio::net::TcpStream;
#[cfg(feature = "embedded-websocket")]
use tokio_util::codec::Framed;
#[cfg(feature = "tungstenite")]
use xrpl::asynch::clients::async_websocket_client::AsyncWebsocketClientTungstenite;
use xrpl::asynch::clients::async_websocket_client::WebsocketOpen;
#[cfg(feature = "embedded-websocket")]
use xrpl::asynch::clients::async_websocket_client::{
    AsyncWebsocketClientEmbeddedWebsocket, EmbeddedWebsocketOptions,
};

mod constants;
pub use constants::*;

#[cfg(feature = "tungstenite")]
pub async fn connect_to_wss_tungstinite_echo(
) -> Result<AsyncWebsocketClientTungstenite<WebsocketOpen>> {
    match ECHO_WSS_SERVER.parse() {
        Ok(url) => match AsyncWebsocketClientTungstenite::open(url).await {
            Ok(websocket) => {
                assert!(websocket.is_open());
                Ok(websocket)
            }
            Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
        },
        Err(err) => Err(anyhow!("Error parsing url: {:?}", err)),
    }
}

#[cfg(feature = "embedded-websocket")]
pub async fn connect_to_ws_embedded_websocket_tokio_echo(
    stream: &mut Framed<TcpStream, codec::Codec>,
    buffer: &mut [u8],
) -> Result<AsyncWebsocketClientEmbeddedWebsocket<rand::rngs::ThreadRng, WebsocketOpen>> {
    let rng = rand::thread_rng();
    let websocket_options = EmbeddedWebsocketOptions {
        path: "/mirror",
        host: "ws.vi-server.org",
        origin: "http://ws.vi-server.org:80",
        sub_protocols: None,
        additional_headers: None,
    };

    match AsyncWebsocketClientEmbeddedWebsocket::open(stream, buffer, rng, &websocket_options).await
    {
        Ok(websocket) => {
            assert!(websocket.is_open());
            Ok(websocket)
        }
        Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
    }
}
