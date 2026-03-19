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

// Pure unit test - no network access needed
#[test]
fn test_url_parsing() {
    let url = url::Url::parse("ws://localhost:6006").unwrap();
    assert_eq!(url.port().unwrap_or(80), 6006);
    assert_eq!(url.host_str().unwrap(), "localhost");
}
