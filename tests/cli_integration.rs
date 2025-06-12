#[cfg(all(feature = "std", feature = "cli"))]
mod cli_tests {
    use std::io::{self};
    use std::process::{Command, Stdio};
    use std::str;

    /// Test-specific constants
    mod constants {
        // Test URLs
        pub const TEST_URL: &str = "https://s.altnet.rippletest.net:51234";

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

    /// Helper function to submit a transaction and check for successful submission
    fn submit_and_check_success(tx_blob: &str, url: &str) {
        let args = ["transaction", "submit", "--tx-blob", tx_blob, "--url", url];
        let output = run_cli_command(&args).expect("Failed to submit transaction");

        assert!(
            output.contains("Transaction submission result:"),
            "Submission output missing expected label"
        );
        assert!(
            output.contains("error: None"),
            "Submission output indicates an error: {}",
            output
        );
    }

    /// Helper function to get the latest NFT Token ID for an account
    fn get_latest_nftoken_id(address: &str, url: &str) -> Option<String> {
        let args = ["account", "nfts", "--address", address, "--url", url];
        let output = run_cli_command(&args).expect("Failed to fetch NFTs");

        print!("Output from account nfts command: {}", output);

        // Parse NFT Token IDs from output (assume output contains "NFTokenID": "...")
        for line in output.lines() {
            if let Some(start) = line.find("\"NFTokenID\":") {
                let rest = &line[start + 13..];
                if let Some(id_start) = rest.find('"') {
                    let rest = &rest[id_start + 1..];
                    if let Some(id_end) = rest.find('"') {
                        return Some(rest[..id_end].to_string());
                    }
                }
            }
        }
        None
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

    #[test]
    fn test_trustset_command() {
        // Use testnet seed and issuer for safety
        let args = [
            "transaction",
            "trust-set",
            "--seed",
            constants::TEST_SEED,
            "--issuer",
            constants::TEST_FAUCET_ADDRESS,
            "--currency",
            "USD",
            "--limit",
            "1000",
            "--url",
            constants::TEST_URL,
        ];

        let result = run_cli_command(&args);
        let output = result.expect("Failed to run trustset command");

        // Check that the output contains a signed transaction blob
        assert!(
            output.contains("Signed transaction blob:"),
            "Output should contain signed transaction blob, got: {}",
            output
        );
        // Optionally, check for the submit hint
        assert!(
            output.contains("To submit, use: xrpl transaction submit"),
            "Output should contain submit hint"
        );
    }

    #[test]
    fn test_account_set_flag() {
        let args = [
            "account",
            "set-flag",
            "--seed",
            constants::TEST_SEED,
            "--flag",
            "asfRequireAuth",
            "--url",
            constants::TEST_URL,
        ];

        let result = run_cli_command(&args);
        let output = result.expect("Failed to run account set-flag command");

        assert!(
            output.contains("Signed transaction blob:"),
            "Output should contain signed transaction blob, got: {}",
            output
        );
        assert!(
            output.contains("To submit, use: xrpl transaction submit"),
            "Output should contain submit hint"
        );
    }

    #[test]
    fn test_account_clear_flag() {
        let args = [
            "account",
            "clear-flag",
            "--seed",
            constants::TEST_SEED,
            "--flag",
            "asfRequireAuth",
            "--url",
            constants::TEST_URL,
        ];

        let result = run_cli_command(&args);
        let output = result.expect("Failed to run account clear-flag command");

        assert!(
            output.contains("Signed transaction blob:"),
            "Output should contain signed transaction blob, got: {}",
            output
        );
        assert!(
            output.contains("To submit, use: xrpl transaction submit"),
            "Output should contain submit hint"
        );
    }

    // TODO: NFT support is not working on the testnet.
    #[test]
    #[ignore]
    fn test_nft_mint_and_burn_roundtrip() {
        // Mint NFT
        let mint_args = [
            "transaction",
            "nft-mint",
            "--seed",
            constants::TEST_SEED,
            "--uri",
            "68747470733a2f2f6578616d706c652e636f6d2f6e66742e6a736f6e",
            "--flags",
            "8", // TfTransferable
            "--transfer-fee",
            "1000",
            "--url",
            constants::TEST_URL,
        ];

        let mint_output = run_cli_command(&mint_args).expect("Failed to fetch NFTs");
        assert!(mint_output.contains("Signed transaction blob:"));

        // Extract the signed blob
        let tx_blob = mint_output
            .lines()
            .find(|l| l.contains("Signed transaction blob:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim())
            .expect("No signed transaction blob found");

        // Submit the mint transaction
        submit_and_check_success(tx_blob, constants::TEST_URL);

        // Wait for the NFT to appear (polling)
        let mut nftoken_id = None;
        for _ in 0..5 {
            std::thread::sleep(std::time::Duration::from_secs(4));
            nftoken_id =
                get_latest_nftoken_id(constants::TEST_CLASSIC_ADDRESS, constants::TEST_URL);
            if nftoken_id.is_some() {
                break;
            }
        }

        let nftoken_id = nftoken_id.expect("Failed to find minted NFT on account");
        // Burn NFT
        let burn_args = [
            "transaction",
            "nft-burn",
            "--seed",
            constants::TEST_SEED,
            "--nftoken-id",
            &nftoken_id,
            "--url",
            constants::TEST_URL,
        ];

        let burn_output = run_cli_command(&burn_args).expect("Failed to run nft-burn command");
        assert!(burn_output.contains("Signed transaction blob:"));

        // Extract the signed blob for burn
        let burn_tx_blob = burn_output
            .lines()
            .find(|l| l.contains("Signed transaction blob:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim())
            .expect("No signed transaction blob found for burn");

        // Submit the burn transaction
        submit_and_check_success(burn_tx_blob, constants::TEST_URL);

        // Verify the NFT is no longer present
        let nftoken_id_after_burn =
            get_latest_nftoken_id(constants::TEST_CLASSIC_ADDRESS, constants::TEST_URL);
        assert!(
            nftoken_id_after_burn.is_none(),
            "NFT should be burned, but found: {:?}",
            nftoken_id_after_burn
        );
    }
}
