use alloc::string::String;
use thiserror_no_std::Error;

#[cfg(feature = "std")]
mod std_cli;

#[cfg(feature = "std")]
pub use std_cli::run;

/// Default mainnet URL for JSON-RPC requests
pub const DEFAULT_MAINNET_URL: &str = "https://xrplcluster.com/";

/// Default testnet URL for faucet functionality
pub const DEFAULT_TESTNET_URL: &str = "https://s.altnet.rippletest.net:51234";

/// Default WebSocket URL
pub const DEFAULT_WEBSOCKET_URL: &str = "wss://xrplcluster.com/";

/// Default limit for paginated results
pub const DEFAULT_PAGINATION_LIMIT: u32 = 10;

// Common CLI commands with conditional derives
#[cfg_attr(feature = "std", derive(clap::Subcommand))]
#[derive(Debug, Clone)]
pub enum Commands {
    /// Generate a new wallet
    GenerateWallet {
        /// Save the wallet to a file
        #[cfg_attr(feature = "std", arg(long))]
        save: bool,
    },
    /// Get wallet info from seed
    WalletFromSeed {
        /// The seed to use
        #[cfg_attr(feature = "std", arg(long))]
        seed: String,
        /// The sequence number
        #[cfg_attr(feature = "std", arg(long, default_value = "0"))]
        sequence: u64,
    },
    /// Get account info
    AccountInfo {
        /// The account address
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
    },
    /// Get current network fee
    GetFee {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
    },
    /// Subscribe to ledger events
    Subscribe {
        /// The XRPL node WebSocket URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_WEBSOCKET_URL.into()))]
        url: String,
        /// Stream type to subscribe to (ledger, transactions, validations)
        #[cfg_attr(feature = "std", arg(long, default_value = "ledger"))]
        stream: String,
        /// Number of events to receive before exiting (0 for unlimited)
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_PAGINATION_LIMIT))]
        limit: u32,
    },
    /// Generate a wallet funded by the testnet faucet
    #[cfg(feature = "std")]
    GenerateFaucetWallet {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_TESTNET_URL.into()))]
        url: String,
    },
    /// Get account transactions
    AccountTx {
        /// The account address
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
        /// Limit the number of transactions returned
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_PAGINATION_LIMIT))]
        limit: u32,
    },
    /// Get server info
    ServerInfo {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
    },
    /// Get ledger data
    LedgerData {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
        /// Ledger index (empty for latest)
        #[cfg_attr(feature = "std", arg(long))]
        ledger_index: Option<String>,
        /// Ledger hash (empty for latest)
        #[cfg_attr(feature = "std", arg(long))]
        ledger_hash: Option<String>,
        /// Limit the number of objects returned
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_PAGINATION_LIMIT as u16))]
        limit: u16,
    },
    /// Get account objects (trust lines, offers, etc.)
    AccountObjects {
        /// The account address
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
        /// Type of objects to return (all, offer, state, etc.)
        #[cfg_attr(feature = "std", arg(long))]
        type_filter: Option<String>,
        /// Limit the number of objects returned
        #[cfg_attr(feature = "std", arg(long, default_value_t = DEFAULT_PAGINATION_LIMIT as u16))]
        limit: u16,
    },
    /// Validate an address
    ValidateAddress {
        /// The address to validate
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
    },
    /// Sign a transaction
    SignTransaction {
        /// The seed to use for signing
        #[cfg_attr(feature = "std", arg(short, long))]
        seed: String,
        /// The transaction type (Payment, AccountSet, etc.)
        #[cfg_attr(feature = "std", arg(short, long))]
        transaction_type: String,
        /// The transaction JSON
        #[cfg_attr(feature = "std", arg(short, long))]
        json: String,
    },
    /// Submit a transaction
    SubmitTransaction {
        /// The signed transaction blob or JSON
        #[cfg_attr(feature = "std", arg(short, long))]
        tx_blob: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(short, long, default_value_t = DEFAULT_MAINNET_URL.into()))]
        url: String,
    },
}

/// Define a custom error type for CLI operations
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Wallet error: {0}")]
    WalletError(#[from] crate::wallet::exceptions::XRPLWalletException),
    #[error("Client error: {0}")]
    ClientError(#[from] crate::asynch::clients::exceptions::XRPLClientException),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Helper error: {0}")]
    HelperError(#[from] crate::asynch::exceptions::XRPLHelperException),
    #[error("Core error: {0}")]
    CoreError(#[from] crate::core::exceptions::XRPLCoreException),
    #[error("Other error: {0}")]
    Other(String),
}

/// Helper function to parse a URL string with proper error handling
#[cfg(feature = "std")]
fn parse_url(url_str: &str) -> Result<url::Url, CliError> {
    url_str.parse().map_err(CliError::UrlParseError)
}

/// Helper function to create a JSON-RPC client with proper error handling
#[cfg(feature = "std")]
fn create_json_rpc_client(
    url_str: &str,
) -> Result<crate::clients::json_rpc::JsonRpcClient, CliError> {
    use crate::clients::json_rpc::JsonRpcClient;
    Ok(JsonRpcClient::connect(parse_url(url_str)?))
}

/// Helper function to handle response display and error conversion
#[cfg(feature = "std")]
fn handle_response<T: core::fmt::Debug>(
    result: Result<T, crate::asynch::clients::exceptions::XRPLClientException>,
    response_type: &str,
) -> Result<(), CliError> {
    match result {
        Ok(response) => {
            alloc::println!("{}: {:#?}", response_type, response);
            Ok(())
        }
        Err(e) => Err(CliError::ClientError(e)),
    }
}

/// Helper function to get or create a Tokio runtime
#[cfg(feature = "std")]
fn get_or_create_runtime() -> Result<tokio::runtime::Runtime, CliError> {
    use tokio::runtime::Runtime;
    Runtime::new().map_err(CliError::IoError)
}

/// Helper function to handle common transaction encoding and output steps
#[cfg(feature = "std")]
fn encode_and_print_tx<T: serde::Serialize>(tx: &T) -> Result<String, CliError> {
    let tx_blob = crate::core::binarycodec::encode(tx)?;
    alloc::println!("Signed transaction blob: {}", tx_blob);
    Ok(tx_blob)
}

pub fn execute_command(command: &Commands) -> Result<(), CliError> {
    match command {
        Commands::GenerateWallet { save } => {
            let wallet = crate::wallet::Wallet::create(None)?;
            alloc::println!("Generated wallet: {:#?}", wallet);
            if *save {
                alloc::println!("Saving wallet functionality not implemented yet");
            }
            Ok(())
        }
        Commands::WalletFromSeed { seed, sequence } => {
            let wallet = crate::wallet::Wallet::new(seed, *sequence)?;
            alloc::println!("Wallet from seed: {:#?}", wallet);
            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::AccountInfo { address, url } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::account_info::AccountInfo;

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let account_info = AccountInfo::new(
                None,                   // id
                address.clone().into(), // account
                None,                   // strict
                None,                   // ledger_index
                None,                   // ledger_hash
                None,                   // queue
                None,                   // signer_lists
            );

            // Execute request and handle response
            handle_response(client.request(account_info.into()), "Account info")
        }
        #[cfg(feature = "std")]
        Commands::GetFee { url } => {
            use crate::ledger::{get_fee, FeeType};

            // Create a runtime and client
            let rt = get_or_create_runtime()?;
            let client = create_json_rpc_client(url)?;

            // Get the current fee within the Tokio runtime
            match rt.block_on(async { get_fee(&client, None, Some(FeeType::Open)) }) {
                Ok(fee) => {
                    alloc::println!("Current network fee: {} drops", fee);
                    Ok(())
                }
                Err(e) => Err(CliError::HelperError(e)),
            }
        }
        #[cfg(feature = "std")]
        Commands::Subscribe { url, stream, limit } => {
            use crate::clients::websocket::WebSocketClient;
            use crate::clients::{SingleExecutorMutex, XRPLSyncWebsocketIO};
            use crate::models::requests::subscribe::{StreamParameter, Subscribe};

            // Parse the stream type
            let stream_param = match stream.to_lowercase().as_str() {
                "ledger" => StreamParameter::Ledger,
                "transactions" => StreamParameter::Transactions,
                "validations" => StreamParameter::Validations,
                _ => return Err(CliError::Other(format!("Unknown stream type: {}", stream))),
            };

            // Open a websocket connection with consistent URL parsing
            let mut websocket: WebSocketClient<SingleExecutorMutex, _> =
                WebSocketClient::open(parse_url(url)?)?;

            // Subscribe to the stream
            let subscribe = Subscribe::new(
                None,
                None,
                None,
                None,
                Some(vec![stream_param]),
                None,
                None,
                None,
            );

            websocket.xrpl_send(subscribe.into())?;

            // Listen for messages
            let mut count = 0;
            loop {
                if *limit > 0 && count >= *limit {
                    break;
                }

                match websocket.xrpl_receive() {
                    Ok(Some(response)) => {
                        alloc::println!("Received: {:#?}", response);
                        count += 1;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        return Err(CliError::ClientError(e));
                    }
                }
            }

            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::GenerateFaucetWallet { url } => {
            use crate::asynch::clients::AsyncJsonRpcClient;
            use crate::asynch::wallet::generate_faucet_wallet;

            // Get or create a runtime
            let rt = get_or_create_runtime()?;

            // Execute within the runtime
            let result = rt.block_on(async {
                let client = AsyncJsonRpcClient::connect(url.parse()?);
                generate_faucet_wallet(&client, None, None, None, None).await
            });

            match result {
                Ok(wallet) => {
                    alloc::println!("Generated faucet wallet: {:#?}", wallet);
                    Ok(())
                }
                Err(e) => Err(CliError::Other(format!(
                    "Failed to generate faucet wallet: {}",
                    e
                ))),
            }
        }
        #[cfg(feature = "std")]
        Commands::AccountTx {
            address,
            url,
            limit,
        } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::account_tx::AccountTx;

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let account_tx = AccountTx::new(
                None,
                address.clone().into(),
                None,
                None,
                None,
                None,
                Some(*limit),
                None,
                None,
                None,
            );

            // Execute request and handle response
            handle_response(client.request(account_tx.into()), "Account transactions")
        }
        #[cfg(feature = "std")]
        Commands::ServerInfo { url } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::server_info::ServerInfo;

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let server_info = ServerInfo::new(None);

            // Execute request and handle response
            handle_response(client.request(server_info.into()), "Server info")
        }
        #[cfg(feature = "std")]
        Commands::LedgerData {
            url,
            ledger_index,
            ledger_hash,
            limit,
        } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::ledger_data::LedgerData;

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let ledger_data = LedgerData::new(
                None,
                None,
                ledger_index.as_deref().map(Into::into),
                ledger_hash.as_deref().map(Into::into),
                Some(*limit),
                None,
            );

            // Execute request and handle response
            handle_response(client.request(ledger_data.into()), "Ledger data")
        }
        #[cfg(feature = "std")]
        Commands::AccountObjects {
            address,
            url,
            type_filter,
            limit,
        } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::account_objects::{AccountObjectType, AccountObjects};
            use std::str::FromStr;

            // Parse the type_filter into AccountObjectType if provided
            let object_type = if let Some(filter) = type_filter.as_deref() {
                match AccountObjectType::from_str(filter) {
                    Ok(obj_type) => Some(obj_type),
                    Err(_) => {
                        return Err(CliError::Other(format!("Invalid object type: {}", filter)))
                    }
                }
            } else {
                None
            };

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let account_objects = AccountObjects::new(
                None,
                address.clone().into(),
                None,
                None,
                object_type,
                None,
                Some(*limit),
                None,
            );

            // Execute request and handle response
            handle_response(client.request(account_objects.into()), "Account objects")
        }
        #[cfg(feature = "std")]
        Commands::ValidateAddress { address } => {
            use crate::core::addresscodec::{is_valid_classic_address, is_valid_xaddress};

            let is_valid_classic = is_valid_classic_address(&address);
            let is_valid_x = is_valid_xaddress(&address);

            if is_valid_classic {
                alloc::println!("Valid classic address: {}", address);
                Ok(())
            } else if is_valid_x {
                use crate::core::addresscodec::xaddress_to_classic_address;
                let (classic_address, tag, is_test) = xaddress_to_classic_address(&address)?;
                alloc::println!("Valid X-address: {}", address);
                alloc::println!("  Classic address: {}", classic_address);
                alloc::println!("  Destination tag: {:?}", tag);
                alloc::println!("  Test network: {}", is_test);
                Ok(())
            } else {
                Err(CliError::Other(format!("Invalid address: {}", address)))
            }
        }
        #[cfg(feature = "std")]
        Commands::SignTransaction {
            seed,
            transaction_type,
            json,
        } => {
            use crate::models::transactions::{
                account_set::AccountSet, offer_cancel::OfferCancel, offer_create::OfferCreate,
                payment::Payment, trust_set::TrustSet,
            };
            use crate::wallet::Wallet;
            use serde_json::Value;

            // Create wallet from seed
            let wallet = Wallet::new(&seed, 0)?;

            // Parse the JSON
            let json_value: Value = serde_json::from_str(&json)?;

            use crate::asynch::transaction::sign;

            // Handle different transaction types
            match transaction_type.to_lowercase().as_str() {
                "payment" => {
                    let mut tx: Payment = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    encode_and_print_tx(&tx)?;
                }
                "accountset" => {
                    let mut tx: AccountSet = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    encode_and_print_tx(&tx)?;
                }
                "offercreate" => {
                    let mut tx: OfferCreate = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    encode_and_print_tx(&tx)?;
                }
                "offercancel" => {
                    let mut tx: OfferCancel = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    encode_and_print_tx(&tx)?;
                }
                "trustset" => {
                    let mut tx: TrustSet = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    encode_and_print_tx(&tx)?;
                }
                _ => {
                    return Err(CliError::Other(format!(
                        "Unsupported transaction type: {}",
                        transaction_type
                    )));
                }
            }

            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::SubmitTransaction { tx_blob, url } => {
            use crate::clients::XRPLSyncClient;
            use crate::models::requests::submit::Submit;

            // Create client with standardized helper function
            let client = create_json_rpc_client(url)?;

            // Create request
            let submit_request = Submit::new(None, tx_blob.into(), None);

            // Execute request and handle response
            handle_response(
                client.request(submit_request.into()),
                "Transaction submission result",
            )
        }
    }
}
