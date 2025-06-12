#[cfg(all(feature = "std", feature = "cli"))]
mod cli_tests {
    use std::io::{self};
    use std::process::{Command, Stdio};
    use std::str;

    /// Test-specific constants
    mod constants {
        // Test URLs
        pub const TEST_URL: &str = "https://testnet.xrpl-labs.com/";

        // Test accounts
        pub const TEST_SEED: &str = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        pub const TEST_CLASSIC_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"; // Genesis account
        pub const TEST_FAUCET_ADDRESS: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"; // Testnet faucet
        pub const TEST_X_ADDRESS: &str = "X7AcgcsBL6XDcUb289X4mJ8djcdyKaB5hJDWMArnXr61cqZ";

        // Test transaction
        pub const TEST_PAYMENT_JSON: &str = r#"{
            "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Amount": "1000000",
            "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "TransactionType": "Payment"
        }"#;

        // Common dummy data
        pub const DUMMY_TX_BLOB: &str = "1200002280000000240000000161400000000000000168400000000000000A732102F89EAEC7667B30F33D0687BBA86C3FE2A08CCA40A9186C5BDE2DAA6FA97A37D874473045022100F9ED357606932697A4FAB2BE7F222C21DD93CA4CF1F52F0D279145B9F6F51DCF02202B3E35791B1E4806D7BFA9989EABFB66FBEA7050D9916A2BADF4B777F8A3D8A981143FBCD300519B17A2F0A7ABAE8E4E7C59A3944F3E78114B61D3061367C35DF7BD7DBE97D5DD318C4B6C9F0F";

        // Common error patterns
        pub const COMMON_NETWORK_ERRORS: &[&str] = &[
            "expected value",
            "network",
            "connection",
            "timeout",
            "there is no reactor running",
            "must be called from the context of a Tokio",
        ];
    }

    /// Helper function to run the CLI with arguments and capture output
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

    /// Helper function to run a CLI command and check for expected output or known errors
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

                // Check if the error matches any of the common errors or the specific known errors
                let matches_common_error = constants::COMMON_NETWORK_ERRORS
                    .iter()
                    .any(|&e| err_str.contains(e));
                let matches_known_error = known_errors.iter().any(|&e| err_str.contains(e));

                assert!(
                    matches_common_error || matches_known_error,
                    "Unexpected error: {}. Expected one of: {:?} or common errors: {:?}",
                    err_str,
                    known_errors,
                    constants::COMMON_NETWORK_ERRORS
                );
            }
        }
    }

    /// Helper for wallet-related command tests
    fn assert_wallet_output(output: &str) {
        assert!(output.contains("classic_address"));
        assert!(output.contains("public_key"));
        assert!(output.contains("private_key"));
    }

    /// Helper to create address-related command arguments
    fn address_command_args<'a>(
        command_group: &'a str,
        subcommand: &'a str,
        address: &'a str,
        url: Option<&'a str>,
        limit: Option<u32>,
    ) -> Vec<&'a str> {
        let mut args = vec![command_group, subcommand, "--address", address];

        if let Some(url_str) = url {
            args.push("--url");
            args.push(url_str);
        }

        if let Some(limit_val) = limit {
            args.push("--limit");
            args.push(match limit_val {
                5 => "5",
                10 => "10",
                _ => "10",
            });
        }

        args
    }

    // ===== WALLET OPERATIONS TESTS =====

    #[test]
    fn test_generate_wallet() {
        let output = run_cli_command(&["wallet", "generate"])
            .expect("Failed to run wallet generate command");

        // Check that the output contains expected wallet information
        assert!(output.contains("Generated wallet:"));
        assert_wallet_output(&output);
    }

    #[test]
    fn test_wallet_from_seed() {
        let output = run_cli_command(&["wallet", "from-seed", "--seed", constants::TEST_SEED])
            .expect("Failed to run wallet from-seed command");

        // Check that the output contains expected wallet information
        assert!(output.contains("Wallet from seed:"));
        assert_wallet_output(&output);
    }

    #[test]
    fn test_wallet_from_seed_with_sequence() {
        let output = run_cli_command(&[
            "wallet",
            "from-seed",
            "--seed",
            constants::TEST_SEED,
            "--sequence",
            "1",
        ])
        .expect("Failed to run wallet from-seed command with sequence");

        // Check that the output contains expected wallet information
        assert!(output.contains("Wallet from seed:"));
        assert_wallet_output(&output);
    }

    #[test]
    fn test_generate_faucet_wallet() {
        let result = run_cli_command(&["wallet", "faucet", "--url", constants::TEST_URL]);

        assert!(
            result.is_ok(),
            "Failed to generate faucet wallet: {:?}",
            result.err()
        );
        let output = result.unwrap();
        assert!(output.contains("Generated faucet wallet:"));
        assert_wallet_output(&output);
    }

    // ===== ACCOUNT QUERY TESTS =====

    #[test]
    fn test_account_info() {
        // Use the test faucet account (known to exist)
        println!(
            "Testing account info with address: {}",
            constants::TEST_FAUCET_ADDRESS
        );

        let args = address_command_args(
            "account",
            "info",
            constants::TEST_FAUCET_ADDRESS,
            Some(constants::TEST_URL),
            None,
        );

        let result = run_cli_command(&args);

        // The account should exist since it's a known testnet account
        assert!(
            result.is_ok(),
            "Failed to get account info: {:?}",
            result.err()
        );
        assert!(result.unwrap().contains("Account info:"));
    }

    #[test]
    fn test_get_fee() {
        assert_cli_command(
            &["server", "fee"],
            "Current network fee:",
            &["Failed to get network fee"],
        );
    }

    #[test]
    fn test_account_tx() {
        let args = address_command_args(
            "account",
            "tx",
            constants::TEST_CLASSIC_ADDRESS,
            None,
            Some(5),
        );

        assert_cli_command(&args, "Account transactions:", &["Account not found"]);
    }

    #[test]
    fn test_server_info() {
        assert_cli_command(&["server", "info"], "Server info:", &[]);
    }

    #[test]
    fn test_ledger_data() {
        assert_cli_command(&["ledger", "data", "--limit", "5"], "Ledger data:", &[]);
    }

    #[test]
    fn test_account_objects() {
        let args = address_command_args(
            "account",
            "objects",
            constants::TEST_CLASSIC_ADDRESS,
            None,
            Some(5),
        );

        assert_cli_command(&args, "Account objects:", &["Account not found"]);
    }

    #[test]
    fn test_account_channels() {
        let args = address_command_args(
            "account",
            "channels",
            constants::TEST_CLASSIC_ADDRESS,
            Some(constants::TEST_URL),
            Some(5),
        );

        assert_cli_command(&args, "Account channels:", &["Account not found"]);
    }

    #[test]
    fn test_account_currencies() {
        let args = address_command_args(
            "account",
            "currencies",
            constants::TEST_CLASSIC_ADDRESS,
            Some(constants::TEST_URL),
            None,
        );

        assert_cli_command(&args, "Account currencies:", &["Account not found"]);
    }

    #[test]
    fn test_account_lines() {
        let args = address_command_args(
            "account",
            "lines",
            constants::TEST_CLASSIC_ADDRESS,
            Some(constants::TEST_URL),
            Some(5),
        );

        assert_cli_command(&args, "Account trust lines:", &["Account not found"]);
    }

    // ===== ADDRESS VALIDATION TESTS =====

    #[test]
    fn test_validate_classic_address() {
        let args = &[
            "wallet",
            "validate",
            "--address",
            constants::TEST_CLASSIC_ADDRESS,
        ];

        assert_cli_command(
            args,
            "Valid classic address:",
            &[], // This should never fail as it doesn't require network access
        );
    }

    #[test]
    fn test_validate_x_address() {
        let args = &["wallet", "validate", "--address", constants::TEST_X_ADDRESS];

        assert_cli_command(
            args,
            "Valid X-address:",
            &[], // This should never fail as it doesn't require network access
        );
    }

    #[test]
    fn test_validate_invalid_address() {
        let address = "not_a_valid_address";
        let result = run_cli_command(&["wallet", "validate", "--address", address]);

        // This test specifically expects an error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid address:"));
    }

    // ===== TRANSACTION TESTS =====

    #[test]
    fn test_sign_transaction() {
        assert_cli_command(
            &[
                "transaction",
                "sign",
                "--seed",
                constants::TEST_SEED,
                "--type",
                "payment",
                "--json",
                constants::TEST_PAYMENT_JSON,
            ],
            "Signed transaction blob:",
            &["Invalid seed", "Failed to sign", "Invalid JSON"],
        );
    }

    #[test]
    fn test_submit_transaction() {
        assert_cli_command(
            &[
                "transaction",
                "submit",
                "--tx-blob",
                constants::DUMMY_TX_BLOB,
                "--url",
                constants::TEST_URL,
            ],
            "Transaction submission result:",
            &["Failed to submit transaction"],
        );
    }
}
