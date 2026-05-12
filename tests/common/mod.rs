#![allow(dead_code)]

pub mod amm;
pub mod constants;
pub mod xchain;

use anyhow::Result;
#[cfg(feature = "std")]
use once_cell::sync::Lazy;
#[cfg(feature = "std")]
use tokio::sync::{Mutex, OnceCell};

#[cfg(all(feature = "websocket", not(feature = "std")))]
use embedded_io_adapters::tokio_1::FromTokio;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use rand::rngs::OsRng;
#[cfg(all(feature = "websocket", not(feature = "std")))]
use tokio::net::TcpStream;
use url::Url;
#[cfg(feature = "websocket")]
use xrpl::asynch::clients::{AsyncWebSocketClient, SingleExecutorMutex, WebSocketOpen};
use xrpl::{asynch::clients::AsyncJsonRpcClient, wallet::Wallet};

/// Genesis account seed (standalone rippled only).
/// Address: rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
#[cfg(feature = "std")]
const GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";

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
// Global mutex to ensure only one test accesses the blockchain at a time
#[cfg(feature = "std")]
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(feature = "std")]
pub async fn get_client() -> &'static AsyncJsonRpcClient {
    CLIENT
        .get_or_init(|| async {
            AsyncJsonRpcClient::connect(Url::parse(constants::STANDALONE_URL).unwrap())
        })
        .await
}

/// Generate a fresh funded wallet by sending 400 XRP from the genesis account.
#[cfg(feature = "std")]
pub async fn generate_funded_wallet() -> Wallet {
    use xrpl::asynch::transaction::sign_and_submit;
    use xrpl::models::transactions::payment::Payment;
    use xrpl::models::{Amount, XRPAmount};

    let genesis = Wallet::new(GENESIS_SEED, 0).expect("genesis wallet");
    let seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
    let new_wallet = Wallet::new(&seed, 0).expect("new wallet");

    let mut payment = Payment::new(
        genesis.classic_address.clone().into(),
        None,                                            // account_txn_id
        None,                                            // fee
        None,                                            // flags
        None,                                            // last_ledger_sequence
        None,                                            // memos
        None,                                            // sequence
        None,                                            // signers
        None,                                            // source_tag
        None,                                            // ticket_sequence
        Amount::XRPAmount(XRPAmount::from("400000000")), // 400 XRP
        new_wallet.classic_address.clone().into(),
        None, // deliver_min
        None, // destination_tag
        None, // invoice_id
        None, // paths
        None, // send_max
    );

    let client = get_client().await;
    sign_and_submit(&mut payment, client, &genesis, true, true)
        .await
        .expect("generate_funded_wallet: funding payment failed");

    ledger_accept().await;
    new_wallet
}

/// Advance the ledger by one close.
#[cfg(feature = "std")]
pub async fn ledger_accept() {
    let _ = reqwest::Client::new()
        .post(constants::STANDALONE_URL)
        .json(&serde_json::json!({"method": "ledger_accept", "params": [{}]}))
        .send()
        .await;
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
                None,
                None,
                None,
                None,
                None,
                None,
                Some("validated".into()),
                None,
                None,
                None,
            )
            .into(),
        )
        .await
        .expect("Failed to get validated ledger");
    let ledger_result: results::ledger::Ledger<'_> =
        response.try_into().expect("Failed to parse ledger result");
    ledger_result.ledger.close_time
}

/// Poll until `close_time >= target`, calling `ledger_accept` each iteration.
#[cfg(feature = "std")]
pub async fn wait_for_ledger_close_time(target: u64) {
    loop {
        if get_ledger_close_time().await >= target {
            return;
        }
        ledger_accept().await;
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

/// Look up the OfferSequence for the first escrow owned by `account`.
/// Mirrors xrpl.js:
///   const accountObjects = (await client.request({command:'account_objects', account})).result.account_objects
///   const sequence = (await client.request({command:'tx', transaction: accountObjects[0].PreviousTxnID})).result.tx_json.Sequence
#[cfg(feature = "std")]
pub async fn get_escrow_offer_sequence(account: &str) -> u32 {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::{
        requests::account_objects::{AccountObjectType, AccountObjects},
        requests::tx::Tx,
        results,
    };

    let client = get_client().await;

    // Step 1: get account_objects and find the escrow entry
    let ao_response = client
        .request(
            AccountObjects::new(
                None,
                account.into(),
                None,
                None,
                Some(AccountObjectType::Escrow),
                None,
                None,
                None,
            )
            .into(),
        )
        .await
        .expect("get_escrow_offer_sequence: account_objects request failed");

    let objects_result: results::account_objects::AccountObjects<'_> = ao_response
        .try_into()
        .expect("get_escrow_offer_sequence: failed to parse account_objects result");

    assert!(
        !objects_result.account_objects.is_empty(),
        "get_escrow_offer_sequence: no escrow objects found for {}",
        account
    );

    let prev_txn_id = objects_result.account_objects[0]["PreviousTxnID"]
        .as_str()
        .expect("PreviousTxnID missing from escrow object")
        .to_string();

    // Step 2: look up the creating tx to get its validated Sequence
    let tx_response = client
        .request(Tx::new(None, None, None, None, Some(prev_txn_id.as_str().into())).into())
        .await
        .expect("get_escrow_offer_sequence: tx request failed");

    let tx_result: results::tx::TxVersionMap<'_> = tx_response
        .try_into()
        .expect("get_escrow_offer_sequence: failed to parse tx result");

    match tx_result {
        results::tx::TxVersionMap::Default(tx) => tx.tx_json["Sequence"]
            .as_u64()
            .expect("Sequence missing in tx_json")
            as u32,
        results::tx::TxVersionMap::V1(tx) => tx.tx_json["Sequence"]
            .as_u64()
            .expect("Sequence missing in tx_json (V1)")
            as u32,
    }
}

/// Sign, submit, assert tesSUCCESS, and advance the ledger.
/// This replaces `submit_and_wait` in all integration tests.
#[cfg(feature = "std")]
pub async fn test_transaction<'a, T, F>(tx: &mut T, wallet: &Wallet)
where
    T: xrpl::models::transactions::Transaction<'a, F>
        + xrpl::models::Model
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + core::fmt::Debug,
    F: strum::IntoEnumIterator + serde::Serialize + core::fmt::Debug + PartialEq + Clone + 'a,
{
    use xrpl::asynch::transaction::sign_and_submit;
    let client = get_client().await;
    let result = sign_and_submit(tx, client, wallet, true, true)
        .await
        .expect("test_transaction: sign_and_submit failed");
    assert_eq!(
        result.engine_result, "tesSUCCESS",
        "Expected tesSUCCESS but got: {} — {}",
        result.engine_result, result.engine_result_message
    );
    ledger_accept().await;
}
