mod constants;

#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
mod tungstenite_clients {
    use super::constants::*;
    use anyhow::anyhow;
    use anyhow::Result;
    use tokio::net::TcpStream;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use xrpl::asynch::clients::AsyncWebsocketClient;
    use xrpl::asynch::clients::{SingleExecutorMutex, WebsocketOpen};

    pub async fn connect_to_wss_tungstinite_test_net<'a>() -> Result<
        AsyncWebsocketClient<
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            SingleExecutorMutex,
            WebsocketOpen,
        >,
    > {
        let stream = connect_async(XRPL_WSS_TEST_NET.to_string())
            .await
            .unwrap()
            .0;
        match AsyncWebsocketClient::open(stream).await {
            Ok(websocket) => {
                // assert!(websocket.is_open());
                Ok(websocket)
            }
            Err(err) => Err(anyhow!("Error connecting to websocket: {:?}", err)),
        }
    }
}

#[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
mod embedded_ws_clients {
    use super::constants::*;
    use anyhow::anyhow;
    use anyhow::Result;
    use std::io;
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;
    use xrpl::asynch::clients::{
        codec::Codec, AsyncWebsocketClient, SingleExecutorMutex, WebsocketOpen,
    };

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
}

#[cfg(all(feature = "embedded-ws", feature = "std", not(feature = "tungstenite")))]
pub use embedded_ws_clients::*;
#[cfg(all(feature = "tungstenite", not(feature = "embedded-ws")))]
pub use tungstenite_clients::*;
