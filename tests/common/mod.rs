#![allow(dead_code)]

pub mod constants;

use anyhow::Result;
#[cfg(feature = "std")]
use once_cell::sync::Lazy;
#[cfg(feature = "std")]
use tokio::sync::{Mutex, OnceCell};

use constants::XRPL_TEST_NET;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use embedded_io_adapters::tokio_1::FromTokio;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use rand::rngs::OsRng;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use tokio::net::TcpStream;
use url::Url;
#[cfg(feature = "websocket")]
use xrpl::asynch::clients::{AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen};
use xrpl::{
    asynch::{clients::AsyncJsonRpcClient, wallet::generate_faucet_wallet},
    wallet::Wallet,
};

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

#[cfg(all(not(feature = "std"), feature = "cli", test))]
pub mod mock_cli;

#[cfg(all(feature = "websocket", feature = "std"))]
pub async fn open_websocket(
    uri: url::Url,
) -> Result<
    xrpl::asynch::clients::AsyncWebSocketClient<
        xrpl::asynch::clients::SingleExecutorMutex,
        xrpl::asynch::clients::WebSocketOpen,
    >,
    Box<dyn std::error::Error>,
> {
    xrpl::asynch::clients::AsyncWebSocketClient::open(uri)
        .await
        .map_err(Into::into)
}

#[cfg(feature = "std")]
static CLIENT: OnceCell<AsyncJsonRpcClient> = OnceCell::const_new();
#[cfg(feature = "std")]
static WALLET: OnceCell<Wallet> = OnceCell::const_new();
// Global mutex to ensure only one test accesses the blockchain at a time
#[cfg(feature = "std")]
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(feature = "std")]
pub async fn get_client() -> &'static AsyncJsonRpcClient {
    CLIENT
        .get_or_init(|| async { AsyncJsonRpcClient::connect(Url::parse(XRPL_TEST_NET).unwrap()) })
        .await
}

#[cfg(feature = "std")]
pub async fn get_wallet() -> &'static Wallet {
    WALLET
        .get_or_init(|| async {
            generate_faucet_wallet(get_client().await, None, None, None, None)
                .await
                .expect("Failed to generate and fund wallet")
        })
        .await
}

// Helper function to run blockchain tests serially
#[cfg(feature = "std")]
pub async fn with_blockchain_lock<F, Fut, T>(f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    // Acquire the mutex to ensure exclusive access
    let _guard = TEST_MUTEX.lock().await;

    // Run the test function
    f().await
}
