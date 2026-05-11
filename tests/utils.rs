//! WebSocket utility integration tests.
//!
//! Tests run against Docker standalone rippled (localhost:6006 for WebSocket).

#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
mod common;

#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
use url::Url;

#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
use crate::common::open_websocket;

/// Local Docker standalone WebSocket endpoint
#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
const STANDALONE_WS_URL: &str = "ws://localhost:6006";

#[tokio::test]
#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
async fn test_open_websocket() {
    let url = Url::parse(STANDALONE_WS_URL).unwrap();
    let result = open_websocket(url).await;
    assert!(
        result.is_ok(),
        "Should successfully open websocket connection to Docker rippled"
    );
}

#[tokio::test]
#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
async fn test_websocket_server_info_request() {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::requests::server_info::ServerInfo;

    let url = Url::parse(STANDALONE_WS_URL).unwrap();
    let client = open_websocket(url).await.expect("open websocket");

    assert!(client.is_open());

    let response = client
        .request(ServerInfo::new(None).into())
        .await
        .expect("server_info request");
    assert!(response.is_success(), "server_info should succeed");

    client.close().await.expect("close websocket");
}

#[tokio::test]
#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
async fn test_websocket_account_info_request_for_genesis() {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::requests::account_info::AccountInfo;
    use xrpl::models::results::account_info::AccountInfoVersionMap;

    let url = Url::parse(STANDALONE_WS_URL).unwrap();
    let client = open_websocket(url).await.expect("open websocket");

    let response = client
        .request(
            AccountInfo::new(
                None,
                crate::common::constants::GENESIS_ACCOUNT.into(),
                None,
                None,
                Some(true),
                None,
                None,
            )
            .into(),
        )
        .await
        .expect("account_info request");
    assert!(response.is_success(), "account_info should succeed");

    let _account_info: AccountInfoVersionMap = response
        .try_into()
        .expect("deserialize AccountInfo result");

    client.close().await.expect("close websocket");
}

#[tokio::test]
#[cfg(all(feature = "integration", feature = "websocket", feature = "std"))]
async fn test_websocket_sequential_requests_on_same_connection() {
    use xrpl::asynch::clients::XRPLAsyncClient;
    use xrpl::models::requests::server_info::ServerInfo;

    let url = Url::parse(STANDALONE_WS_URL).unwrap();
    let client = open_websocket(url).await.expect("open websocket");

    for _ in 0..3 {
        let response = client
            .request(ServerInfo::new(None).into())
            .await
            .expect("server_info request");
        assert!(response.is_success());
    }

    client.close().await.expect("close websocket");
}

// Pure unit test - no network access needed
#[test]
fn test_url_parsing() {
    let url = url::Url::parse("ws://localhost:6006").unwrap();
    assert_eq!(url.port().unwrap_or(80), 6006);
    assert_eq!(url.host_str().unwrap(), "localhost");
}
