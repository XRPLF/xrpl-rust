use core::convert::TryFrom;

use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::models::{
    amount::XRPAmount, results::exceptions::XRPLResultException, XRPLModelException,
    XRPLModelResult,
};

use super::XRPLResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerState<'a> {
    pub state: State<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State<'a> {
    /// The version number of the running rippled version.
    pub build_version: Cow<'a, str>,
    /// Range expression indicating the sequence numbers of the ledger versions the local rippled has in
    /// its database. It is possible to be a disjoint sequence, e.g. "2500-5000,32570-7695432". If the server
    /// does not have any complete ledgers (for example, it recently started syncing with the network),
    /// this is the string empty.
    pub complete_ledgers: Cow<'a, str>,
    /// Amount of time spent waiting for I/O operations, in milliseconds. If this number is not very, very low,
    /// then the rippled server is probably having serious load issues.
    pub io_latency_ms: u32,
    /// The number of times this server has had over 250 transactions waiting to be processed at once. A large number
    /// here may mean that your server is unable to handle the transaction load of the XRP Ledger network. For detailed
    /// recommendations of future-proof server specifications, see Capacity Planning.
    pub jq_trans_overflow: JqTransOverflow<'a>,
    /// Information about the last time the server closed a ledger, including the amount of time it took to reach
    /// a consensus and the number of trusted validators participating.
    pub last_close: LastClose,
    /// The baseline amount of server load used in transaction cost calculations. If the `load_factor` is equal to
    /// the `load_base`, then only the base transaction cost is enforced. If the `load_factor` is higher than the
    /// `load_base`, then transaction costs are multiplied by the ratio between them. For example, if the `load_factor`
    /// is double the `load_base`, then transaction costs are doubled.
    pub load_base: u32,
    /// The load factor the server is currently enforcing. The ratio between this value and the `load_base` determines
    /// the multiplier for transaction costs. The load factor is determined by the highest of the individual
    /// server's load factor, the cluster's load factor, the open ledger cost, and the overall network's load factor.
    pub load_factor: u32,
    /// A list of ports where the server is listening for API commands. Each entry in the array is a Port Descriptor object.
    pub ports: Vec<PortDescriptor<'a>>,
    /// Public key used to verify this server for peer-to-peer communications. This node key pair is automatically generated
    /// by the server the first time it starts up. (If deleted, the server can create a new pair of keys.) You can set a
    /// persistent value in the config file using the `[node_seed]` config option, which is useful for clustering.
    pub pubkey_node: Cow<'a, str>,
    /// (Admin only) Public key used by this node to sign ledger validations. This validation key pair is derived from
    /// the `[validator_token]` or `[validation_seed]` config field.
    pub pubkey_validator: Cow<'a, str>,
    /// A string indicating to what extent the server is participating in the network. See Possible Server States for more details.
    pub server_state: Cow<'a, str>,
    /// The number of consecutive microseconds the server has been in the current state.
    pub server_state_duration_us: u32,
    /// A map of various server states with information about the time the server spends in each. This can be useful for tracking
    /// the long-term health of your server's connectivity to the network.
    pub state_accounting: RippledServerState<'a>,
    /// The current time in UTC, according to the server's clock.
    pub time: Cow<'a, str>,
    /// Number of consecutive seconds that the server has been operational.
    pub uptime: u32,
    /// Minimum number of trusted validations required to validate a ledger version. Some circumstances may cause the server to
    /// require more validations.
    pub validation_quorum: u32,
    /// If true, this server is amendment blocked. If the server is not amendment blocked,
    /// the response omits this field.
    pub amendment_blocked: Option<bool>,
    /// Information on the most recently closed ledger that has not been validated by consensus.
    /// If the most recently validated ledger is available, the response omits this field and
    /// includes `validated_ledger` instead. The member fields are the same as the `validated_ledger` field.
    pub closed_ledger: Option<ValidatedLedger<'a>>,
    /// (Admin only) Detailed information about the current load state of the server.
    pub load: Option<Load<'a>>,
    /// The current multiplier to the transaction cost to get into the open ledger, in fee levels.
    pub load_factor_fee_escalation: Option<u32>,
    /// The current multiplier to the transaction cost to get into the queue, if the queue is full, in fee levels.
    pub load_factor_fee_queue: Option<u32>,
    /// The transaction cost with no load scaling, in fee levels.
    pub load_factor_fee_reference: Option<u32>,
    /// The load factor the server is enforcing, based on load to the server, cluster, and network, but not
    /// factoring in the open ledger cost.
    pub load_factor_server: Option<u32>,
    /// The id of the network the server is connected to.
    pub network_id: Option<u32>,
    /// (Omitted by reporting mode servers) How many other rippled servers this one is currently connected to.
    pub peers: Option<u32>,
    /// (Reporting mode servers only) Information about this server's reporting-mode specific configurations.
    pub reporting: Option<Reporting<'a>>,
    /// Information about the most recent fully-validated ledger. If the most recent validated ledger is not
    /// available, the response omits this field and includes `closed_ledger` instead.
    pub validated_ledger: Option<ValidatedLedger<'a>>,
    /// (Admin only) When the current validator list expires, in seconds since the Ripple Epoch, or 0 if the server
    /// has yet to load a published validator list.
    pub validator_list_expires: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JqTransOverflow<'a> {
    Int(u32),
    String(Cow<'a, str>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LastClose {
    /// The amount of time it took to reach a consensus on the most recently validated
    /// ledger version, in milliseconds.
    pub converge_time: u32,
    /// How many trusted validators the server considered (including itself, if configured as a validator)
    /// in the consensus process for the most recently validated ledger version.
    pub proposers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortDescriptor<'a> {
    pub port: Port<'a>,
    pub protocol: Vec<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Port<'a> {
    Int(u32),
    String(Cow<'a, str>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reporting<'a> {
    /// A list of P2P-mode servers this reporting mode is retrieving data from. Each entry in this array is an ETL Source object.
    pub etl_sources: Vec<ETLSource<'a>>,
    /// If true, this server is writing to the external database with ledger data. If false, it is not currently writing,
    /// possibly because another reporting mode server is currently populating a shared database, or because it's
    /// configured as read-only.
    pub is_writer: bool,
    /// An ISO 8601 timestamp indicating when this server last saw a new validated ledger from any of its P2P mode sources.
    pub last_publish_time: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ETLSource<'a> {
    pub connected: bool,
    pub grpc_port: Cow<'a, str>,
    pub ip: Cow<'a, str>,
    pub last_message_arrival_time: Cow<'a, str>,
    pub validated_ledgers_range: Cow<'a, str>,
    pub websocket_port: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Load<'a> {
    /// Information about the rate of different types of jobs the server is doing and how much time it spends on each.
    pub job_types: Vec<Cow<'a, str>>,
    /// The number of threads in the server's main job pool.
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RippledServerState<'a> {
    /// The server is not connected to the XRP Ledger peer-to-peer network whatsoever. It may be running in offline mode,
    /// or it may not be able to access the network for whatever reason.
    pub disconnected: StateAccounting<'a>,
    /// The server believes it is connected to the network.
    pub connected: StateAccounting<'a>,
    /// The server is currently behind on ledger versions. (It is normal for a server to spend a few minutes catching
    /// up after you start it.)
    pub syncing: StateAccounting<'a>,
    /// The server is in agreement with the network.
    pub tracking: StateAccounting<'a>,
    /// The server is fully caught-up with the network and could participate in validation, but is not doing so
    /// (possibly because it has not been configured as a validator).
    pub full: StateAccounting<'a>,
    /// The server is currently participating in validation of the ledger.
    pub validating: StateAccounting<'a>,
    /// The server is participating in validation of the ledger and currently proposing its own version.
    pub proposing: StateAccounting<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateAccounting<'a> {
    /// The number of microseconds the server has spent in this state. (This is updated whenever the server transitions into another state.)
    pub duration_us: Cow<'a, str>,
    /// The number of times the server has changed into this state.
    pub transitions: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatedLedger<'a> {
    /// Base fee, in drops of XRP, for propagating a transaction to the network.
    pub base_fee: XRPAmount<'a>,
    /// Time this ledger was closed, in seconds since the Ripple Epoch.
    pub close_time: u32,
    /// Unique hash of this ledger version, as hexadecimal.
    pub hash: Cow<'a, str>,
    /// The minimum account reserve, as of the most recent validated ledger version.
    pub reserve_base: XRPAmount<'a>,
    /// The owner reserve for each item an account owns, as of the most recent validated ledger version.
    pub reserve_inc: XRPAmount<'a>,
    /// The ledger index of the most recently validated ledger version.
    pub seq: u32,
}

impl<'a> TryFrom<XRPLResult<'a>> for ServerState<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::ServerState(server_state) => Ok(server_state),
            res => Err(XRPLResultException::UnexpectedResultType(
                "ServerState".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
