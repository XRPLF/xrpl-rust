pub mod codec;

use anyhow::anyhow;
use anyhow::Result;

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use tokio::net::TcpStream;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use tokio_util::codec::Framed;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
use xrpl::asynch::clients::AsyncWebsocketClient;
use xrpl::asynch::clients::WebsocketOpen;
#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
use xrpl::asynch::clients::{AsyncWebsocketClient, EmbeddedWebsocketOptions};

mod constants;
pub use constants::*;

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub async fn connect_to_wss_tungstinite_echo() -> Result<AsyncWebsocketClient<WebsocketOpen>> {
    match ECHO_WSS_SERVER.parse() {
        Ok(url) => match AsyncWebsocketClient::open(url).await {
            Ok(websocket) => {
                assert!(websocket.is_open());
                Ok(websocket)
            }
            Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
        },
        Err(err) => Err(anyhow!("Error parsing url: {:?}", err)),
    }
}

#[cfg(all(feature = "embedded-ws", not(feature = "tungstenite")))]
pub async fn connect_to_ws_embedded_websocket_tokio_echo(
    stream: &mut Framed<TcpStream, codec::Codec>,
    buffer: &mut [u8],
) -> Result<AsyncWebsocketClient<rand::rngs::ThreadRng, WebsocketOpen>> {
    let rng = rand::thread_rng();
    let websocket_options = EmbeddedWebsocketOptions {
        path: "/mirror",
        host: "ws.vi-server.org",
        origin: "http://ws.vi-server.org:80",
        sub_protocols: None,
        additional_headers: None,
    };

    match AsyncWebsocketClient::open(stream, buffer, rng, &websocket_options).await {
        Ok(websocket) => {
            assert!(websocket.is_open());
            Ok(websocket)
        }
        Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
    }
}
