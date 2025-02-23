use alloc::{borrow::Cow, vec::Vec};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response format for the server_info command, which returns various
/// information about the rippled server's current state and configuration.
///
/// See Server Info:
/// `<https://xrpl.org/server_info.html#server_info>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ServerInfo<'a> {
    /// If true, this server is amendment blocked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amendment_blocked: Option<bool>,

    /// The version number of the running rippled server
    pub build_version: Cow<'a, str>,

    /// Information about the most recently closed ledger that has not been
    /// validated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_ledger: Option<LedgerInfo<'a>>,

    /// Range expression indicating the sequence numbers of the ledger versions
    /// in the database
    pub complete_ledgers: Cow<'a, str>,

    /// Performance metrics for RPC calls and JobQueue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<Value>,

    /// Items currently being run in the job queue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_activity: Option<Value>,

    /// Server hostname or RFC-1751 word based on node public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostid: Option<Cow<'a, str>>,

    /// Amount of time spent waiting for I/O operations, in milliseconds
    pub io_latency_ms: u32,

    /// Number of times server had over 250 transactions waiting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jq_trans_overflow: Option<Cow<'a, str>>,

    /// Information about the last ledger close
    pub last_close: LastClose,

    /// Detailed information about the current load state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<Load>,

    /// Current transaction cost multiplier
    pub load_factor: u32,

    /// Transaction cost multiplier based on local load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_local: Option<u32>,

    /// Transaction cost multiplier from network load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_net: Option<u32>,

    /// Transaction cost multiplier from cluster load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_cluster: Option<u32>,

    /// Transaction cost multiplier for open ledger
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<u32>,

    /// Transaction cost multiplier for queue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<u32>,

    /// Transaction cost multiplier excluding open ledger
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<u32>,

    /// Number of connected peer servers
    pub peers: u32,

    /// List of ports listening for API commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<Value>>,

    /// Public key for peer-to-peer communications
    pub pubkey_node: Cow<'a, str>,

    /// Public key for ledger validations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_validator: Option<Cow<'a, str>>,

    /// Reporting mode configuration information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting: Option<Reporting<'a>>,

    /// Current server state
    pub server_state: Cow<'a, str>,

    /// Microseconds in current state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_state_duration_us: Option<u64>,

    /// Server state accounting information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_accounting: Option<Value>,

    /// Current UTC time according to server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Cow<'a, str>>,

    /// Seconds server has been operational
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,

    /// Information about the most recent validated ledger
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_ledger: Option<LedgerInfo<'a>>,

    /// Minimum required trusted validations
    pub validation_quorum: u32,

    /// Validator list expiration time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_list_expires: Option<u32>,

    /// Validator list information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validator_list: Option<ValidatorList<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidatorList<'a> {
    pub count: u32,
    pub expiration: u32,
    pub status: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LastClose {
    /// Time to reach consensus in seconds
    pub converge_time_s: u64,
    /// Number of trusted validators considered
    pub proposers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Load {
    /// Information about job types and time spent
    pub job_types: Vec<Value>,
    /// Number of threads in main job pool
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Reporting<'a> {
    /// List of P2P-mode servers
    pub etl_sources: Vec<Value>,
    /// Whether server is writing to external database
    pub is_writer: bool,
    /// Last publish time
    pub last_publish_time: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerInfo<'a> {
    /// Time since ledger close in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// Base fee in XRP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_xrp: Option<u64>,
    /// Unique ledger hash
    pub hash: Cow<'a, str>,
    /// Minimum XRP reserve for accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve_base_xrp: Option<u64>,
    /// Additional XRP reserve per owned object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve_inc_xrp: Option<u64>,
    /// Ledger sequence number
    pub seq: u32,
}
