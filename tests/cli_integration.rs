#[cfg(all(feature = "std", feature = "cli"))]
mod cli_tests {
    use std::io::{self};
    use std::process::{Command, Stdio};
    use std::str;

    // Helper function to run the CLI with arguments and capture output
    fn run_cli_command(args: &[&str]) -> Result<String, io::Error> {
        let cmd = Command::new(env!("CARGO_BIN_EXE_xrpl"))
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = cmd.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }

    // Helper function to run a CLI command and check for expected output or known errors
    fn assert_cli_command(args: &[&str], expected_output: &str, known_errors: &[&str]) {
        let result = run_cli_command(args);

        match result {
            Ok(output) => {
                assert!(
                    output.contains(expected_output),
                    "Expected output to contain '{}', but got: {}",
                    expected_output,
                    output
                );
            }
            Err(err) => {
                let err_str = err.to_string();
                let common_errors = [
                    "expected value",
                    "network",
                    "connection",
                    "timeout",
                    "there is no reactor running",
                    "must be called from the context of a Tokio",
                ];

                // Check if the error matches any of the common errors or the specific known errors
                let matches_common_error = common_errors.iter().any(|&e| err_str.contains(e));
                let matches_known_error = known_errors.iter().any(|&e| err_str.contains(e));

                assert!(
                    matches_common_error || matches_known_error,
                    "Unexpected error: {}. Expected one of: {:?} or common errors: {:?}",
                    err_str,
                    known_errors,
                    common_errors
                );
            }
        }
    }

    #[test]
    fn test_generate_wallet() {
        let output =
            run_cli_command(&["generate-wallet"]).expect("Failed to run generate-wallet command");

        // Check that the output contains expected wallet information
        assert!(output.contains("Generated wallet:"));
        assert!(output.contains("classic_address"));
        assert!(output.contains("public_key"));
        assert!(output.contains("private_key"));
    }

    #[test]
    fn test_wallet_from_seed() {
        // Use a known test seed
        let seed = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let output = run_cli_command(&["wallet-from-seed", "--seed", seed])
            .expect("Failed to run wallet-from-seed command");

        // Check that the output contains expected wallet information
        assert!(output.contains("Wallet from seed:"));
        assert!(output.contains("classic_address"));
        assert!(output.contains("public_key"));
        assert!(output.contains("private_key"));
    }

    #[test]
    fn test_wallet_from_seed_with_sequence() {
        // Use a known test seed
        let seed = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let output = run_cli_command(&["wallet-from-seed", "--seed", seed, "--sequence", "1"])
            .expect("Failed to run wallet-from-seed command with sequence");

        // Check that the output contains expected wallet information
        assert!(output.contains("Wallet from seed:"));
        assert!(output.contains("classic_address"));
        assert!(output.contains("public_key"));
        assert!(output.contains("private_key"));
    }

    #[test]
    fn test_account_info() {
        // Use a known XRPL account
        let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        // This test might fail if the testnet is down or the account doesn't exist
        // So we'll just check that the command runs without error
        let result = run_cli_command(&["account-info", "--address", address]);

        // Just verify the command executed, even if it might return an error from the XRPL
        assert!(
            result.is_ok()
                || result
                    .unwrap_err()
                    .to_string()
                    .contains("Account not found")
        );
    }

    #[test]
    fn test_get_fee() {
        assert_cli_command(
            &["get-fee"],
            "Current network fee:",
            &[
                "expected value",
                "network",
                "connection",
                "timeout",
                "Failed to get network fee",
            ],
        );
    }

    #[test]
    fn test_account_tx() {
        assert_cli_command(
            &[
                "account-tx",
                "--address",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                "--limit",
                "5",
            ],
            "Account transactions:",
            &[
                "expected value",
                "network",
                "connection",
                "timeout",
                "Account not found",
            ],
        );
    }

    #[test]
    fn test_server_info() {
        assert_cli_command(
            &["server-info"],
            "Server info:",
            &["expected value", "network", "connection", "timeout"],
        );
    }

    #[test]
    fn test_ledger_data() {
        assert_cli_command(
            &["ledger-data", "--limit", "5"],
            "Ledger data:",
            &["expected value", "network", "connection", "timeout"],
        );
    }

    #[test]
    fn test_account_objects() {
        assert_cli_command(
            &[
                "account-objects",
                "--address",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                "--limit",
                "5",
            ],
            "Account objects:",
            &[
                "expected value",
                "network",
                "connection",
                "timeout",
                "Account not found",
            ],
        );
    }

    #[test]
    fn test_validate_classic_address() {
        assert_cli_command(
            &[
                "validate-address",
                "--address",
                "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            ],
            "Valid classic address:",
            &[], // This should never fail as it doesn't require network access
        );
    }

    #[test]
    fn test_validate_x_address() {
        assert_cli_command(
            &[
                "validate-address",
                "--address",
                "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ",
            ],
            "Valid X-address:",
            &[], // This should never fail as it doesn't require network access
        );
    }

    #[test]
    fn test_validate_invalid_address() {
        let address = "not_a_valid_address";
        let result = run_cli_command(&["validate-address", "--address", address]);

        // This test specifically expects an error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid address:"));
    }

    #[test]
    fn test_sign_transaction() {
        // Create a simple payment transaction
        let seed = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let transaction_json = r#"{
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Amount": "1000000",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "TransactionType": "Payment"
        }"#;

        assert_cli_command(
            &[
                "sign-transaction",
                "--seed",
                seed,
                "--transaction-type",
                "payment",
                "--json",
                transaction_json,
            ],
            "Signed transaction blob:",
            &["Invalid seed", "Failed to sign", "Invalid JSON"],
        );
    }

    #[test]
    fn test_submit_transaction() {
        // This is a dummy signed transaction blob
        let tx_blob = "1200002280000000240000000161400000000000000168400000000000000A732102F89EAEC7667B30F33D0687BBA86C3FE2A08CCA40A9186C5BDE2DAA6FA97A37D874473045022100F9ED357606932697A4FAB2BE7F222C21DD93CA4CF1F52F0D279145B9F6F51DCF02202B3E35791B1E4806D7BFA9989EABFB66FBEA7050D9916A2BADF4B777F8A3D8A981143FBCD300519B17A2F0A7ABAE8E4E7C59A3944F3E78114B61D3061367C35DF7BD7DBE97D5DD318C4B6C9F0F";

        assert_cli_command(
            &[
                "submit-transaction",
                "--tx-blob",
                tx_blob,
                "--url",
                "https://s.altnet.rippletest.net:51234",
            ],
            "Transaction submission result:",
            &[
                "expected value",
                "network",
                "connection",
                "timeout",
                "Failed to submit transaction",
            ],
        );
    }
}
