pub mod codec;

use xrpl::asynch::clients::async_websocket_client::WebsocketOpen;
#[cfg(feature = "tungstenite")]
use xrpl::asynch::clients::async_websocket_client::AsyncWebsocketClientTungstenite;
#[cfg(feature = "embedded-websocket")]
use xrpl::asynch::clients::async_websocket_client::{
    AsyncWebsocketClientEmbeddedWebsocket, EmbeddedWebsocketOptions,
};
#[cfg(feature = "embedded-websocket")]
use tokio_util::codec::Framed;
#[cfg(feature = "embedded-websocket")]
use tokio::net::TcpStream;

mod constants;
pub use constants::*;

#[cfg(feature = "tungstenite")]
pub async fn connect_to_wss_tungstinite_echo() -> AsyncWebsocketClientTungstenite<WebsocketOpen> {
    let websocket = AsyncWebsocketClientTungstenite::open(ECHO_WSS_SERVER.parse().unwrap())
        .await
        .unwrap();
    assert!(websocket.is_open());

    websocket
}

#[cfg(feature = "embedded-websocket")]
pub async fn connect_to_ws_embedded_websocket_tokio_echo(
    stream: &mut Framed<TcpStream, codec::Codec>,
    buffer: &mut [u8],
) -> AsyncWebsocketClientEmbeddedWebsocket<rand::rngs::ThreadRng, WebsocketOpen> {
    let rng = rand::thread_rng();
    let websocket_options = EmbeddedWebsocketOptions {
        path: "/mirror",
        host: "ws.vi-server.org",
        origin: "http://ws.vi-server.org:80",
        sub_protocols: None,
        additional_headers: None,
    };

    let websocket =
        AsyncWebsocketClientEmbeddedWebsocket::open(stream, buffer, rng, &websocket_options)
            .await
            .unwrap();

    websocket
}
