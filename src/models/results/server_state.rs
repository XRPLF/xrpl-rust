use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::amount::XRPAmount;

/// Server state response data
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct State<'a> {
    /// If true, this server is amendment blocked
    pub amendment_blocked: Option<bool>,
    /// The version number of the running rippled version
    pub build_version: Cow<'a, str>,
    /// Range expression indicating the sequence numbers of the ledger versions
    /// in the database. Can be disjoint, e.g. "2500-5000,32570-7695432"
    pub complete_ledgers: Option<Cow<'a, str>>,
    /// Information on the most recently closed ledger that has not been
    /// validated
    pub closed_ledger: Option<ValidatedLedger<'a>>,
    /// Amount of time spent waiting for I/O operations, in milliseconds
    pub io_latency_ms: Option<u32>,
    /// Number of times server had over 250 transactions waiting to be processed
    pub jq_trans_overflow: Option<Cow<'a, str>>,
    /// Information about the last time the server closed a ledger
    pub last_close: Option<LastClose>,
    /// Baseline amount of server load used in transaction cost calculations
    pub load_base: Option<u32>,
    /// Current load factor the server is enforcing
    pub load_factor: Option<u32>,
    /// Current multiplier to the transaction cost to get into the open ledger
    pub load_factor_fee_escalation: Option<u32>,
    /// Current multiplier to the transaction cost to get into the queue
    pub load_factor_fee_queue: Option<u32>,
    /// Transaction cost with no load scaling
    pub load_factor_fee_reference: Option<u32>,
    /// Load factor based on load to server, cluster, and network
    pub load_factor_server: Option<u32>,
    /// Count of peer disconnections
    pub peer_disconnects: Option<Cow<'a, str>>,
    /// Count of resource-related peer disconnections
    pub peer_disconnects_resources: Option<Cow<'a, str>>,
    /// Number of other rippled servers currently connected
    pub peers: Option<u32>,
    /// Public key used for peer-to-peer communications
    pub pubkey_node: Option<Cow<'a, str>>,
    /// Current server state (e.g., "full", "validating", etc.)
    pub server_state: Option<Cow<'a, str>>,
    /// Consecutive microseconds in current state
    pub server_state_duration_us: Option<Cow<'a, str>>,
    /// Information about time spent in various server states
    pub state_accounting: Option<StateAccounting>,
    /// Current UTC time according to server
    pub time: Option<Cow<'a, str>>,
    /// Number of consecutive seconds server has been operational
    pub uptime: Option<u64>,
    /// Information about the most recent fully-validated ledger
    pub validated_ledger: Option<ValidatedLedger<'a>>,
    /// Minimum number of trusted validations required
    pub validation_quorum: Option<u32>,
    /// List of ports where the server is listening for API commands
    pub ports: Option<Cow<'a, [PortDescriptor<'a>]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LastClose {
    pub converge_time: u32,
    pub proposers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StateAccountingEntry {
    pub duration_us: Cow<'static, str>,
    pub transitions: Cow<'static, str>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StateAccounting {
    pub connected: Option<StateAccountingEntry>,
    pub disconnected: Option<StateAccountingEntry>,
    pub full: Option<StateAccountingEntry>,
    pub syncing: Option<StateAccountingEntry>,
    pub tracking: Option<StateAccountingEntry>,
}

/// Port configuration information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortDescriptor<'a> {
    /// Port number where the server is listening
    pub port: Cow<'a, str>,
    /// List of protocols being served on this port
    pub protocol: Cow<'a, [Cow<'a, str>]>,
}

/// Information about a validated ledger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidatedLedger<'a> {
    /// Base fee in drops of XRP for transaction propagation
    pub base_fee: XRPAmount<'a>,
    /// Time this ledger was closed (seconds since Ripple Epoch)
    pub close_time: u32,
    /// Unique hash of this ledger version
    pub hash: Cow<'a, str>,
    /// Minimum account reserve
    pub reserve_base: XRPAmount<'a>,
    /// Owner reserve for each owned item
    pub reserve_inc: XRPAmount<'a>,
    /// Ledger index of this version
    pub seq: u32,
}

/// Response format for the server_state command, which requests a
/// human-readable version of various information about the rippled server's
/// current state.
///
/// See Server State:
/// `<https://xrpl.org/server_state.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ServerState<'a> {
    pub state: State<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_state_deserialize() {
        let json = r#"{
            "state": {
                "build_version": "1.7.2",
                "complete_ledgers": "65844785-65887184",
                "io_latency_ms": 3,
                "jq_trans_overflow": "580",
                "last_close": {
                    "converge_time": 3012,
                    "proposers": 41
                },
                "load_base": 256,
                "load_factor": 134022,
                "load_factor_fee_escalation": 134022,
                "load_factor_fee_queue": 256,
                "load_factor_fee_reference": 256,
                "load_factor_server": 256,
                "peer_disconnects": "792367",
                "peer_disconnects_resources": "7273",
                "peers": 72,
                "pubkey_node": "n9LNvsFiYfFf8va6pma2PHGJKVLSyZweN1iBAkJQSeHw4GjM8gvN",
                "server_state": "full",
                "server_state_duration_us": "422128665555",
                "state_accounting": {
                    "connected": {
                        "duration_us": "172799714",
                        "transitions": "1"
                    },
                    "disconnected": {
                        "duration_us": "309059",
                        "transitions": "1"
                    },
                    "full": {
                        "duration_us": "6020429212246",
                        "transitions": "143"
                    },
                    "syncing": {
                        "duration_us": "413813232",
                        "transitions": "152"
                    },
                    "tracking": {
                        "duration_us": "266553605",
                        "transitions": "152"
                    }
                },
                "time": "2021-Aug-24 20:43:43.043406 UTC",
                "uptime": 6021282,
                "validated_ledger": {
                    "base_fee": 10,
                    "close_time": 683153020,
                    "hash": "ABEF3D24015E8B6B7184B4ABCEDC0E0E3AA4F0677FAB91C40B1E500707C1F3E5",
                    "reserve_base": 20000000,
                    "reserve_inc": 5000000,
                    "seq": 65887184
                },
                "validation_quorum": 33
            },
            "status": "success"
        }"#;

        let result: ServerState = serde_json::from_str(json).unwrap();

        assert_eq!(result.state.build_version, "1.7.2");
        assert_eq!(
            result.state.complete_ledgers,
            Some("65844785-65887184".into())
        );
        assert_eq!(result.state.io_latency_ms, Some(3));
        assert_eq!(result.state.jq_trans_overflow, Some("580".into()));
        assert_eq!(
            result.state.last_close.as_ref().unwrap().converge_time,
            3012
        );
        assert_eq!(result.state.last_close.as_ref().unwrap().proposers, 41);
        assert_eq!(result.state.load_base, Some(256));
        assert_eq!(result.state.load_factor, Some(134022));
        assert_eq!(result.state.peers, Some(72));
        assert_eq!(result.state.server_state, Some("full".into()));

        let validated_ledger = result.state.validated_ledger.unwrap();
        assert_eq!(validated_ledger.base_fee, XRPAmount::from("10"));
        assert_eq!(validated_ledger.close_time, 683153020);
        assert_eq!(
            validated_ledger.hash,
            "ABEF3D24015E8B6B7184B4ABCEDC0E0E3AA4F0677FAB91C40B1E500707C1F3E5"
        );
        assert_eq!(validated_ledger.seq, 65887184);

        let state_accounting = result.state.state_accounting.unwrap();
        assert_eq!(state_accounting.full.as_ref().unwrap().transitions, "143");
        assert_eq!(
            state_accounting.syncing.as_ref().unwrap().transitions,
            "152"
        );
    }
}
