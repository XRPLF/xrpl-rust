//! Test utilities for XRPL Rust library
//!
//! This module provides common utilities for testing, including:
//! - Test wallet creation
//! - Common test patterns
//! - Timeout helpers

use alloc::string::{String, ToString};
use core::time::Duration;

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

/// Result of a test operation.
#[derive(Debug)]
pub enum TestResult<T> {
    /// Test completed successfully
    Success(T),
    /// Test failed with an unexpected error
    Failed(String),
}

/// Handle the test result and return the value if successful, or fail the test.
#[macro_export]
macro_rules! handle_test_result {
    ($result:expr, $test_name:expr) => {
        match $result {
            $crate::utils::testing::TestResult::Success(value) => value,
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

    /// Create a failed result with an error message
    pub fn failed(error: impl Into<String>) -> Self {
        Self::Failed(error.into())
    }

    /// Convert a Result into a TestResult, preserving errors as failures.
    pub fn from_result(result: Result<T, impl ToString>) -> Self {
        match result {
            Ok(value) => Self::Success(value),
            Err(error) => Self::Failed(error.to_string()),
        }
    }

    /// Handle the test result appropriately (pass or panic)
    pub fn handle(self, test_name: &str) {
        match self {
            Self::Success(_) => {}
            Self::Failed(error) => {
                panic!("❌ {} failed: {}", test_name, error);
            }
        }
    }
}

/// Helper for testing network operations with timeout handling.
#[cfg(feature = "tokio-rt")]
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
        Err(_) => TestResult::Failed(alloc::format!("{} timed out", operation_name)),
    }
}

/// Test wallet credentials for deterministic testing
#[cfg(feature = "wallet")]
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
    pub const TESTNET_URL: &str = "https://s.altnet.rippletest.net:51234/";
    pub const ALT_TESTNET_URL: &str = "https://faucet.altnet.rippletest.net:443";
}

/// Assertion helpers for common test patterns
pub mod assertions {
    use crate::models::transactions::Transaction;
    use core::fmt::Debug;
    use strum::IntoEnumIterator;

    /// Assert that a transaction is properly signed
    pub fn assert_transaction_signed<'a, T, U>(tx: &T)
    where
        T: Transaction<'a, U>,
        U: Clone + Debug + PartialEq + serde::Serialize + IntoEnumIterator,
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
        U: Clone + Debug + PartialEq + serde::Serialize + IntoEnumIterator,
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
        U: Clone + Debug + PartialEq + serde::Serialize + IntoEnumIterator,
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
    #[cfg(feature = "wallet")]
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
    fn test_result_handling() {
        // Test success case
        let result = TestResult::success("test_value");
        match result {
            TestResult::Success(value) => assert_eq!(value, "test_value"),
            _ => panic!("Expected success"),
        }

        // Test from_result with error
        let other_error: Result<(), &str> = Err("validation failed");
        let result = TestResult::from_result(other_error);
        match result {
            TestResult::Failed(_) => {} // Expected
            _ => panic!("Expected failure for error"),
        }
    }

    #[test]
    fn test_result_failed_constructor_and_from_err() {
        match TestResult::<()>::failed("boom") {
            TestResult::Failed(error) => assert_eq!(error, "boom"),
            TestResult::Success(_) => panic!("expected failed result"),
        }

        match TestResult::<()>::from_result(Err("nope")) {
            TestResult::Failed(error) => assert_eq!(error, "nope"),
            TestResult::Success(_) => panic!("expected failed result"),
        }
    }

    #[test]
    fn test_result_from_ok_and_success_handle() {
        match TestResult::<u8>::from_result(Ok::<u8, &str>(7)) {
            TestResult::Success(value) => assert_eq!(value, 7),
            TestResult::Failed(error) => panic!("unexpected failure: {error}"),
        }

        TestResult::success(()).handle("success should not panic");
    }

    #[test]
    #[should_panic(expected = "❌ expected-panic failed: nope")]
    fn test_result_handle_failed_panics() {
        TestResult::<()>::failed("nope").handle("expected-panic");
    }

    #[cfg(feature = "wallet")]
    #[test]
    fn test_wallet_creation() {
        let wallet = test_wallets::create_test_wallet().unwrap();
        assertions::assert_valid_wallet(&wallet);
    }
}
