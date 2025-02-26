mod common;

use url::Url;

use crate::common::open_websocket;

#[tokio::test]
#[cfg(feature = "websocket")]
async fn test_open_websocket() {
    let url = Url::parse("wss://s.altnet.rippletest.net:51233").unwrap();
    let result = open_websocket(url).await;
    assert!(
        result.is_ok(),
        "Should successfully open websocket connection"
    );
}

#[tokio::test]
#[cfg(all(feature = "websocket", not(feature = "std")))]
async fn test_open_websocket_no_std() {
    let url = Url::parse("wss://s.altnet.rippletest.net:51233").unwrap();
    let result = open_websocket(url).await;
    assert!(
        result.is_ok(),
        "Should successfully open websocket connection in no_std environment"
    );
}

#[test]
fn test_url_parsing() {
    let url = Url::parse("wss://s.altnet.rippletest.net:51233").unwrap();
    assert_eq!(url.port().unwrap_or(80), 51233);
    assert_eq!(url.host_str().unwrap(), "s.altnet.rippletest.net");
}
