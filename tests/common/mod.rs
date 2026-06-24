#![allow(dead_code)]

pub mod amm;
pub mod constants;
pub mod payment;
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
#[cfg(all(feature = "websocket", not(feature = "std")))]
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
    let host = uri.host_str().expect("open_websocket: URI has no host");
    let url = format!("{host}:{port}");

    let tcp = TcpStream::connect(&url)
        .await
        .expect("open_websocket: TcpStream::connect failed");
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
///
/// Panics if the HTTP round-trip fails so test failures are surfaced
/// immediately rather than silently proceeding with a stale ledger.
#[cfg(feature = "std")]
pub async fn ledger_accept() {
    reqwest::Client::new()
        .post(constants::STANDALONE_URL)
        .json(&serde_json::json!({"method": "ledger_accept", "params": [{}]}))
        .send()
        .await
        .expect("ledger_accept: HTTP request failed")
        .error_for_status()
        .expect("ledger_accept: server returned error status");
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
/// Queries account_objects to find the escrow, then looks up its creating
/// transaction to extract the validated Sequence number.
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

/// Parameters for [`submit_tx`] — use struct literal syntax so each argument
/// is self-documenting at call sites.
#[cfg(feature = "std")]
pub struct SubmitOptions<'w> {
    pub wallet: &'w Wallet,
    /// Auto-fill sequence, fee, and other transaction fields before signing.
    pub autofill: bool,
    /// Validate that the fee satisfies the network's minimum requirement.
    pub check_fee: bool,
}

/// Submit a transaction without asserting success. Returns the raw
/// `engine_result` string so callers can assert specific `tec`/`tem` codes.
///
/// Use [`test_transaction`] instead when you expect `tesSUCCESS`.
#[cfg(feature = "std")]
pub async fn submit_tx<'a, T, F>(tx: &mut T, opts: SubmitOptions<'_>) -> String
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
    sign_and_submit(tx, client, opts.wallet, opts.autofill, opts.check_fee)
        .await
        .expect("submit_tx: sign_and_submit failed")
        .engine_result
        .to_string()
}

/// Provision an accepted Credential and return its on-chain hash (ledger index).
///
/// Submits CredentialCreate (issuer → subject) then CredentialAccept (subject),
/// reads the resulting Credential ledger object and returns its `index` field.
#[cfg(feature = "std")]
pub async fn provision_credential(
    issuer: &xrpl::wallet::Wallet,
    subject: &xrpl::wallet::Wallet,
    credential_type: &str,
) -> String {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::{
        requests::account_objects::{AccountObjectType, AccountObjects},
        results,
        transactions::{
            credential_accept::CredentialAccept, credential_create::CredentialCreate, CommonFields,
            TransactionType,
        },
    };

    let client = get_client().await;

    let mut create = CredentialCreate {
        common_fields: CommonFields {
            account: issuer.classic_address.clone().into(),
            transaction_type: TransactionType::CredentialCreate,
            ..Default::default()
        },
        subject: subject.classic_address.clone().into(),
        credential_type: credential_type.to_owned().into(),
        ..Default::default()
    };
    test_transaction(&mut create, issuer).await;

    let mut accept = CredentialAccept {
        common_fields: CommonFields {
            account: subject.classic_address.clone().into(),
            transaction_type: TransactionType::CredentialAccept,
            ..Default::default()
        },
        issuer: issuer.classic_address.clone().into(),
        credential_type: credential_type.to_owned().into(),
    };
    test_transaction(&mut accept, subject).await;

    let ao_resp = client
        .request(
            AccountObjects::new(
                None,
                subject.classic_address.clone().into(),
                None,
                None,
                Some(AccountObjectType::Credential),
                None,
                None,
                None,
            )
            .into(),
        )
        .await
        .expect("provision_credential: account_objects request failed");
    let ao_result: results::account_objects::AccountObjects<'_> = ao_resp
        .try_into()
        .expect("provision_credential: parse account_objects");
    assert!(
        !ao_result.account_objects.is_empty(),
        "provision_credential: no credential object found after CredentialAccept"
    );
    ao_result.account_objects[0]["index"]
        .as_str()
        .expect("provision_credential: index field missing on credential object")
        .to_string()
}

/// Set up credential-based DepositPreauth: provision an accepted Credential
/// (issuer → subject), then authorize it on `destination`. Returns the
/// credential hash so callers can attach it to `credential_ids` on transactions.
#[cfg(feature = "std")]
pub async fn provision_credential_for_destination(
    issuer: &xrpl::wallet::Wallet,
    subject: &xrpl::wallet::Wallet,
    destination: &xrpl::wallet::Wallet,
    credential_type: &str,
) -> String {
    use xrpl::models::{
        transactions::{deposit_preauth::DepositPreauth, CommonFields, TransactionType},
        CredentialAuthorization, CredentialAuthorizationFields,
    };

    let credential_hash = provision_credential(issuer, subject, credential_type).await;

    // Enable Deposit Authorization on destination so rippled enforces DepositPreauth rules.
    // Without lsfDepositAuth set, rippled ignores DepositPreauth entries entirely and any
    // payment flows through regardless of credential_ids.
    {
        use xrpl::models::transactions::account_set::{AccountSet, AccountSetFlag};
        let mut acct_set = AccountSet {
            common_fields: CommonFields {
                account: destination.classic_address.clone().into(),
                transaction_type: TransactionType::AccountSet,
                ..Default::default()
            },
            set_flag: Some(AccountSetFlag::AsfDepositAuth),
            ..Default::default()
        };
        test_transaction(&mut acct_set, destination).await;
    }

    let creds = vec![CredentialAuthorization::new(
        CredentialAuthorizationFields::new(
            issuer.classic_address.clone().into(),
            credential_type.to_owned().into(),
        ),
    )];

    let mut preauth = DepositPreauth {
        common_fields: CommonFields {
            account: destination.classic_address.clone().into(),
            transaction_type: TransactionType::DepositPreauth,
            ..Default::default()
        },
        authorize_credentials: Some(creds),
        ..Default::default()
    };
    test_transaction(&mut preauth, destination).await;

    credential_hash
}
