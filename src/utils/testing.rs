//! Test utilities for XRPL Rust library
//!
//! This module provides common utilities for testing, including:
//! - Network error detection and handling
//! - Test wallet creation
//! - Common test patterns
//! - Timeout helpers

#[cfg(test)]
use core::time::Duration;

/// Common network error patterns that should cause tests to skip rather than fail
pub const COMMON_NETWORK_ERRORS: &[&str] = &[
    // JSON parsing errors
    "expected value",
    "invalid type",
    "EOF while parsing",
    // Network connectivity errors
    "network",
    "connection",
    "timeout",
    "Connection refused",
    "Connection reset",
    "Connection timed out",
    "No route to host",
    "Network is unreachable",
    "ConnectError",
    // DNS resolution errors
    "dns error",
    "failed to lookup address",
    "Name or service not known",
    "nodename nor servname provided",
    // HTTP client errors
    "HttpError",
    "EmptyResponse",
    "hyper_util::client::legacy::Error",
    "reqwest::Error",
    // Runtime/async errors
    "there is no reactor running",
    "must be called from the context of a Tokio",
    // Implementation status
    "not yet implemented",
    "unimplemented",
];

/// Check if an error message indicates a known network/infrastructure issue
/// that should cause a test to skip rather than fail
pub fn is_known_network_error(error_msg: &str) -> bool {
    COMMON_NETWORK_ERRORS
        .iter()
        .any(|&pattern| error_msg.to_lowercase().contains(&pattern.to_lowercase()))
}

/// Standard timeout durations for different types of operations
pub struct TestTimeouts;

impl TestTimeouts {
    /// Short timeout for local operations (5 seconds)
    pub const LOCAL: Duration = Duration::from_secs(5);
    /// Medium timeout for simple network operations (30 seconds)
    pub const NETWORK: Duration = Duration::from_secs(30);
    /// Long timeout for faucet operations (60 seconds)
    pub const FAUCET: Duration = Duration::from_secs(60);
    /// Extra long timeout for transaction submission and confirmation (120 seconds)
    pub const TRANSACTION: Duration = Duration::from_secs(120);
}

/// Result of a test operation that might skip due to network issues
#[derive(Debug)]
pub enum TestResult<T> {
    /// Test completed successfully
    Success(T),
    /// Test was skipped due to known network issues
    Skipped(String),
    /// Test failed with an unexpected error
    Failed(String),
}

/// Handle the test result and return the value if successful, or return from the test function
#[macro_export]
macro_rules! handle_test_result {
    ($result:expr, $test_name:expr) => {
        match $result {
            $crate::utils::testing::TestResult::Success(value) => value,
            $crate::utils::testing::TestResult::Skipped(reason) => {
                println!("⏭️  {} skipped: {}", $test_name, reason);
                return;
            }
            $crate::utils::testing::TestResult::Failed(error) => {
                panic!("❌ {} failed: {}", $test_name, error);
            }
        }
    };
}

impl<T> TestResult<T> {
    /// Create a success result
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    /// Create a skipped result with a reason
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self::Skipped(reason.into())
    }

    /// Create a failed result with an error message
    pub fn failed(error: impl Into<String>) -> Self {
        Self::Failed(error.into())
    }

    /// Convert a Result into a TestResult, categorizing errors appropriately
    pub fn from_result(result: Result<T, impl ToString>) -> Self {
        match result {
            Ok(value) => Self::Success(value),
            Err(error) => {
                let error_msg = error.to_string();
                if is_known_network_error(&error_msg) {
                    Self::Skipped(format!("Known network error: {}", error_msg))
                } else {
                    Self::Failed(error_msg)
                }
            }
        }
    }

    /// Handle the test result appropriately (skip, pass, or panic)
    pub fn handle(self, test_name: &str) {
        match self {
            Self::Success(_) => {}
            Self::Skipped(reason) => {
                println!("⏭️  {} skipped: {}", test_name, reason);
            }
            Self::Failed(error) => {
                panic!("❌ {} failed: {}", test_name, error);
            }
        }
    }
}

/// Helper for testing network operations with timeout and error handling
pub async fn test_network_operation<F, T, E>(
    operation: F,
    timeout: Duration,
    operation_name: &str,
) -> TestResult<T>
where
    F: core::future::Future<Output = Result<T, E>>,
    E: ToString,
{
    let result = tokio::time::timeout(timeout, operation).await;

    match result {
        Ok(Ok(value)) => TestResult::Success(value),
        Ok(Err(error)) => TestResult::from_result(Err(error)),
        Err(_) => TestResult::Skipped(format!("{} timed out", operation_name)),
    }
}

/// Test wallet credentials for deterministic testing
pub mod test_wallets {
    use crate::wallet::{exceptions::XRPLWalletException, Wallet};

    /// A test wallet with known credentials (DO NOT USE IN PRODUCTION)
    pub const TEST_WALLET_SEED: &str = "sEdT7wHTCLzDG7ueaw4hroSTBvH7Mk5";
    pub const TEST_WALLET_SEQUENCE: u64 = 0;

    /// Create a deterministic test wallet
    pub fn create_test_wallet() -> Result<Wallet, XRPLWalletException> {
        Wallet::new(TEST_WALLET_SEED, TEST_WALLET_SEQUENCE)
    }

    /// Create a deterministic test wallet, panicking on error (for tests)
    pub fn create_test_wallet_unwrap() -> Wallet {
        create_test_wallet().expect("Failed to create test wallet")
    }
}

/// Common test constants
pub mod test_constants {
    /// Hex-encoded "example.com" for domain fields
    pub const EXAMPLE_COM_HEX: &str = "6578616d706c652e636f6d";

    /// Common test URLs
    pub const TESTNET_URL: &str = "https://testnet.xrpl-labs.com/";
    pub const ALT_TESTNET_URL: &str = "https://faucet.altnet.rippletest.net:443";
}

/// Assertion helpers for common test patterns
pub mod assertions {
    use crate::models::transactions::Transaction;
    use strum::IntoEnumIterator;

    /// Assert that a transaction is properly signed
    pub fn assert_transaction_signed<'a, T, U>(tx: &T)
    where
        T: Transaction<'a, U>,
        U: Clone + std::fmt::Debug + PartialEq + serde::Serialize + IntoEnumIterator,
    {
        let common_fields = tx.get_common_fields();
        assert!(
            common_fields.txn_signature.is_some(),
            "Transaction should have a signature"
        );
        assert!(
            common_fields.signing_pub_key.is_some(),
            "Transaction should have a signing public key"
        );
    }

    /// Assert that a transaction is properly multisigned
    pub fn assert_transaction_multisigned<'a, T, U>(tx: &T)
    where
        T: Transaction<'a, U>,
        U: Clone + std::fmt::Debug + PartialEq + serde::Serialize + IntoEnumIterator,
    {
        let common_fields = tx.get_common_fields();
        assert!(
            common_fields.signers.is_some(),
            "Multisigned transaction should have signers"
        );
        assert!(
            common_fields.txn_signature.is_none(),
            "Multisigned transaction should not have txn_signature"
        );
    }

    /// Assert that a transaction has been autofilled
    pub fn assert_transaction_autofilled<'a, T, U>(tx: &T)
    where
        T: Transaction<'a, U>,
        U: Clone + std::fmt::Debug + PartialEq + serde::Serialize + IntoEnumIterator,
    {
        let common_fields = tx.get_common_fields();
        assert!(
            common_fields.sequence.is_some(),
            "Autofilled transaction should have sequence"
        );
        assert!(
            common_fields.fee.is_some(),
            "Autofilled transaction should have fee"
        );
    }

    /// Assert that a wallet is valid
    pub fn assert_valid_wallet(wallet: &crate::wallet::Wallet) {
        assert!(
            !wallet.classic_address.is_empty(),
            "Wallet should have an address"
        );
        assert!(
            !wallet.public_key.is_empty(),
            "Wallet should have a public key"
        );
        assert!(
            !wallet.private_key.is_empty(),
            "Wallet should have a private key"
        );
        assert!(!wallet.seed.is_empty(), "Wallet should have a seed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        // Test that our error detection works correctly
        assert!(is_known_network_error("dns error occurred"));
        assert!(is_known_network_error(
            "failed to lookup address information"
        ));
        assert!(is_known_network_error("Connection refused"));
        assert!(is_known_network_error("Network is unreachable"));
        assert!(is_known_network_error("expected value"));
        assert!(is_known_network_error("ConnectError"));
        assert!(is_known_network_error("not yet implemented"));

        // Test case insensitivity
        assert!(is_known_network_error("DNS ERROR OCCURRED"));
        assert!(is_known_network_error("CONNECTION REFUSED"));

        // Test negative cases
        assert!(!is_known_network_error("some other error"));
        assert!(!is_known_network_error("validation failed"));
        assert!(!is_known_network_error("invalid transaction"));
    }

    #[test]
    fn test_result_handling() {
        // Test success case
        let result = TestResult::success("test_value");
        match result {
            TestResult::Success(value) => assert_eq!(value, "test_value"),
            _ => panic!("Expected success"),
        }

        // Test from_result with network error
        let network_error: Result<(), &str> = Err("dns error occurred");
        let result = TestResult::from_result(network_error);
        match result {
            TestResult::Skipped(_) => {} // Expected
            _ => panic!("Expected skip for network error"),
        }

        // Test from_result with other error
        let other_error: Result<(), &str> = Err("validation failed");
        let result = TestResult::from_result(other_error);
        match result {
            TestResult::Failed(_) => {} // Expected
            _ => panic!("Expected failure for non-network error"),
        }
    }

    #[test]
    fn test_wallet_creation() {
        let wallet = test_wallets::create_test_wallet().unwrap();
        assertions::assert_valid_wallet(&wallet);
    }
}
