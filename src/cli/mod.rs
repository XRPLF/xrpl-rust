use alloc::string::String;
use thiserror_no_std::Error;

#[cfg(feature = "std")]
mod std_cli;

#[cfg(feature = "std")]
pub use std_cli::run;

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
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
    },
    /// Get current network fee
    GetFee {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
    },
    /// Subscribe to ledger events
    Subscribe {
        /// The XRPL node WebSocket URL
        #[cfg_attr(feature = "std", arg(long, default_value = "wss://xrplcluster.com/"))]
        url: String,
        /// Stream type to subscribe to (ledger, transactions, validations)
        #[cfg_attr(feature = "std", arg(long, default_value = "ledger"))]
        stream: String,
        /// Number of events to receive before exiting (0 for unlimited)
        #[cfg_attr(feature = "std", arg(long, default_value = "10"))]
        limit: u32,
    },
    /// Generate a faucet wallet (testnet only)
    GenerateFaucetWallet {
        /// The XRPL testnet node URL
        #[cfg_attr(
            feature = "std",
            arg(long, default_value = "https://testnet.xrpl-labs.com/")
        )]
        url: String,
    },
    /// Get account transactions
    AccountTx {
        /// The account address
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
        /// Limit the number of transactions returned
        #[cfg_attr(feature = "std", arg(long, default_value = "10"))]
        limit: u32,
    },
    /// Get server info
    ServerInfo {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
    },
    /// Get ledger data
    LedgerData {
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
        /// Ledger index (empty for latest)
        #[cfg_attr(feature = "std", arg(long))]
        ledger_index: Option<String>,
        /// Ledger hash (empty for latest)
        #[cfg_attr(feature = "std", arg(long))]
        ledger_hash: Option<String>,
        /// Limit the number of objects returned
        #[cfg_attr(feature = "std", arg(long, default_value = "10"))]
        limit: u16,
    },
    /// Get account objects (trust lines, offers, etc.)
    AccountObjects {
        /// The account address
        #[cfg_attr(feature = "std", arg(long))]
        address: String,
        /// The XRPL node URL
        #[cfg_attr(feature = "std", arg(long, default_value = "https://xrplcluster.com/"))]
        url: String,
        /// Type of objects to return (all, offer, state, etc.)
        #[cfg_attr(feature = "std", arg(long))]
        type_filter: Option<String>,
        /// Limit the number of objects returned
        #[cfg_attr(feature = "std", arg(long, default_value = "10"))]
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
        #[cfg_attr(
            feature = "std",
            arg(short, long, default_value = "https://xrplcluster.com/")
        )]
        url: String,
    },
}

// Define a custom error type for CLI operations
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
            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::account_info::AccountInfo;
            use tokio::runtime::Runtime;

            // Create a new Tokio runtime
            let rt = Runtime::new()?;

            // Connect to the XRPL node
            let client = JsonRpcClient::connect(url.parse()?);

            // Create the account info request
            let account_info = AccountInfo::new(
                None,                   // id
                address.clone().into(), // account
                None,                   // strict
                None,                   // ledger_index
                None,                   // ledger_hash
                None,                   // queue
                None,                   // signer_lists
            );

            // Execute the request within the Tokio runtime
            match rt.block_on(async { client.request(account_info.into()) }) {
                Ok(response) => {
                    alloc::println!("Account info: {:#?}", response);
                    Ok(())
                }
                Err(e) => {
                    // Provide a more user-friendly error message
                    Err(CliError::Other(format!(
                        "Failed to get account info: {}",
                        e
                    )))
                }
            }
        }
        #[cfg(feature = "std")]
        Commands::GetFee { url } => {
            use crate::clients::json_rpc::JsonRpcClient;
            use crate::ledger::{get_fee, FeeType};
            use tokio::runtime::Runtime;

            // Create a new Tokio runtime
            let rt = Runtime::new()?;
            // Connect to the XRPL node
            let client = JsonRpcClient::connect(url.parse()?);

            // Get the current fee within the Tokio runtime
            match rt.block_on(async { get_fee(&client, None, Some(FeeType::Open)) }) {
                Ok(fee) => {
                    alloc::println!("Current network fee: {} drops", fee);
                    Ok(())
                }
                Err(e) => {
                    // Convert the error to a more user-friendly message
                    Err(CliError::Other(format!("Failed to get network fee: {}", e)))
                }
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

            // Open a websocket connection
            let mut websocket: WebSocketClient<SingleExecutorMutex, _> =
                WebSocketClient::open(url.parse()?)?;

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
            use tokio::runtime::Runtime;

            // Create a runtime
            let rt = Runtime::new()?;

            // Connect to the testnet
            let client = AsyncJsonRpcClient::connect(url.parse()?);

            // Generate a faucet wallet
            let wallet = rt.block_on(generate_faucet_wallet(&client, None, None, None, None))?;

            alloc::println!("Generated faucet wallet: {:#?}", wallet);
            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::AccountTx {
            address,
            url,
            limit,
        } => {
            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::account_tx::AccountTx;

            let client = JsonRpcClient::connect(url.parse()?);
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
            let response = client.request(account_tx.into())?;
            alloc::println!("Account transactions: {:#?}", response);
            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::ServerInfo { url } => {
            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::server_info::ServerInfo;

            let client = JsonRpcClient::connect(url.parse()?);
            let server_info = ServerInfo::new(None);
            let response = client.request(server_info.into())?;
            alloc::println!("Server info: {:#?}", response);
            Ok(())
        }
        #[cfg(feature = "std")]
        Commands::LedgerData {
            url,
            ledger_index,
            ledger_hash,
            limit,
        } => {
            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::ledger_data::LedgerData;

            let client = JsonRpcClient::connect(url.parse()?);
            let ledger_data = LedgerData::new(
                None,
                None,
                ledger_index.as_deref().map(Into::into),
                ledger_hash.as_deref().map(Into::into),
                Some(*limit),
                None,
            );

            match client.request(ledger_data.into()) {
                Ok(response) => {
                    alloc::println!("Ledger data: {:#?}", response);
                    Ok(())
                }
                Err(e) => {
                    // Provide more context for the error
                    Err(CliError::Other(format!(
                        "Failed to retrieve ledger data: {}",
                        e
                    )))
                }
            }
        }
        #[cfg(feature = "std")]
        Commands::AccountObjects {
            address,
            url,
            type_filter,
            limit,
        } => {
            use std::str::FromStr;

            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::account_objects::AccountObjectType;
            use crate::models::requests::account_objects::AccountObjects;

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

            let client = JsonRpcClient::connect(url.parse()?);
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
            let response = client.request(account_objects.into())?;
            alloc::println!("Account objects: {:#?}", response);
            Ok(())
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
            use crate::asynch::transaction::sign;
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

            // Handle different transaction types
            // TODO Write a converter
            match transaction_type.to_lowercase().as_str() {
                "payment" => {
                    let mut tx: Payment = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    let tx_blob = crate::core::binarycodec::encode(&tx)?;
                    alloc::println!("Signed transaction blob: {}", tx_blob);
                }
                "accountset" => {
                    let mut tx: AccountSet = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    let tx_blob = crate::core::binarycodec::encode(&tx)?;
                    alloc::println!("Signed transaction blob: {}", tx_blob);
                }
                "offercreate" => {
                    let mut tx: OfferCreate = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    let tx_blob = crate::core::binarycodec::encode(&tx)?;
                    alloc::println!("Signed transaction blob: {}", tx_blob);
                }
                "offercancel" => {
                    let mut tx: OfferCancel = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    let tx_blob = crate::core::binarycodec::encode(&tx)?;
                    alloc::println!("Signed transaction blob: {}", tx_blob);
                }
                "trustset" => {
                    let mut tx: TrustSet = serde_json::from_value(json_value)?;
                    sign(&mut tx, &wallet, false)?;
                    let tx_blob = crate::core::binarycodec::encode(&tx)?;
                    alloc::println!("Signed transaction blob: {}", tx_blob);
                }
                // Add more transaction types as needed
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
            use crate::clients::{json_rpc::JsonRpcClient, XRPLSyncClient};
            use crate::models::requests::submit::Submit;

            // Create a client
            let client = JsonRpcClient::connect(url.parse()?);

            // Create a submit request
            let submit_request = Submit::new(None, tx_blob.into(), None);

            // Submit the transaction
            match client.request(submit_request.into()) {
                Ok(response) => {
                    alloc::println!("Transaction submission result: {:#?}", response);
                    Ok(())
                }
                Err(e) => Err(CliError::Other(format!(
                    "Failed to submit transaction: {}",
                    e
                ))),
            }
        }
    }
}
