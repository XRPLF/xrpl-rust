#![allow(dead_code)]

pub mod amm;
pub mod constants;
pub mod xchain;

use anyhow::Result;
#[cfg(feature = "std")]
use once_cell::sync::Lazy;
#[cfg(feature = "std")]
use tokio::sync::{Mutex, OnceCell};

#[cfg(feature = "std")]
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

/// Generate a fresh funded wallet on every call via the testnet faucet.
/// Use this for tests that modify wallet state (flags, trust lines, signers, balances).
/// Prefer `get_wallet()` only for simple read-only or non-state-mutating submissions.
#[cfg(feature = "std")]
pub async fn generate_funded_wallet() -> Wallet {
    generate_faucet_wallet(get_client().await, None, None, None, None)
        .await
        .expect("Failed to generate and fund wallet")
}

/// Advance the ledger by one close.
/// No-op on testnet — ledgers close automatically every ~3–4 seconds.
/// Will be replaced with an actual `ledger_accept` RPC call when switching to Docker standalone mode.
#[cfg(feature = "std")]
pub async fn ledger_accept() {
    // Intentional no-op for testnet. Docker standalone mode requires:
    //   get_client().await.request(LedgerAccept::new()).await.expect("ledger_accept failed");
}

/// Return the `close_time` of the most-recent validated ledger in Ripple epoch seconds.
#[cfg(feature = "std")]
pub async fn get_ledger_close_time() -> u64 {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::{requests::ledger::Ledger, results};
    let client = get_client().await;
    let response = client
        .request(
            Ledger::new(
                None, None, None, None, None, None,
                Some("validated".into()),
                None, None, None,
            )
            .into(),
        )
        .await
        .expect("Failed to get validated ledger");
    let ledger_result: results::ledger::Ledger<'_> = response
        .try_into()
        .expect("Failed to parse ledger result");
    ledger_result.ledger.close_time
}

/// Poll the validated ledger until `close_time >= target`, sleeping 4 s between polls.
/// On testnet ledgers close automatically; on Docker standalone ledger_accept() drives time.
#[cfg(feature = "std")]
pub async fn wait_for_ledger_close_time(target: u64) {
    loop {
        if get_ledger_close_time().await >= target {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    }
}

/// Serialize all blockchain-mutating tests to prevent sequence number conflicts.
#[cfg(feature = "std")]
pub async fn with_blockchain_lock<F, Fut, T>(f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let _guard = TEST_MUTEX.lock().await;
    f().await
}
