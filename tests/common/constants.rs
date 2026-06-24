#![allow(dead_code)]

pub const ECHO_WS_SERVER: &'static str = "ws://ws.vi-server.org/mirror";
pub const ECHO_WSS_SERVER: &'static str = "wss://ws.vi-server.org/mirror";

pub const XRPL_TEST_NET: &'static str = "https://testnet.xrpl-labs.com/";
pub const XRPL_WSS_TEST_NET: &'static str = "wss://testnet.xrpl-labs.com/";
pub const XRPL_WS_TEST_NET: &'static str = "wss://s.altnet.rippletest.net:51233/";

pub const GENESIS_ACCOUNT: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

/// HTTP JSON-RPC endpoint for local Docker standalone rippled.
pub const STANDALONE_URL: &str = "http://localhost:5005";

/// "chainlink" ASCII hex-encoded (Provider Blob field).
pub const ORACLE_PROVIDER: &str = "636861696E6C696E6B";

/// "currency" ASCII hex-encoded (AssetClass Blob field).
pub const ORACLE_ASSET_CLASS: &str = "63757272656E6379";

/// Short opaque URI hex-encoded ("did_example").
pub const ORACLE_URI: &str = "6469645F6578616D706C65";

/// "https://example.com" ASCII hex-encoded (used in construction-only tests).
pub const ORACLE_URI_HTTPS: &str = "68747470733A2F2F6578616D706C652E636F6D";

// ---------------------------------------------------------------------------
// Credentials / XLS-70 test fixtures
// ---------------------------------------------------------------------------

/// "KYC" ASCII hex-encoded. Used as credential_type in all credential integration tests.
pub const CREDENTIAL_TYPE_KYC: &str = "4B5943";
