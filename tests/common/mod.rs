use anyhow::Result;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use embedded_io_adapters::tokio_1::FromTokio;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use rand::rngs::OsRng;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use tokio::net::TcpStream;
use url::Url;
#[cfg(feature = "websocket")]
use xrpl::asynch::clients::{AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen};

#[cfg(all(feature = "websocket", not(feature = "std")))]
pub async fn open_websocket(
    uri: Url,
) -> Result<
    AsyncWebSocketClient<4096, FromTokio<TcpStream>, OsRng, SingleExecutorMutex, WebSocketOpen>,
> {
    use anyhow::anyhow;

    let port = uri.port().unwrap_or(80);
    let url = format!("{}:{}", uri.host_str().unwrap(), port);

    let tcp = TcpStream::connect(&url).await.unwrap();
    let stream = FromTokio::new(tcp);
    let rng = OsRng;
    match AsyncWebSocketClient::open(stream, uri, rng, None, None).await {
        Ok(client) => Ok(client),
        Err(e) => Err(anyhow!(e)),
    }
}

#[cfg(all(feature = "websocket", feature = "std"))]
pub async fn open_websocket(
    uri: Url,
) -> Result<AsyncWebSocketClient<SingleExecutorMutex, WebSocketOpen>> {
    AsyncWebSocketClient::open(uri).await.map_err(Into::into)
}
