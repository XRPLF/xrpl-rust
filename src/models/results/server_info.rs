use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::Amount;

/// Server information
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Info<'a> {
    /// If true, this server is amendment blocked
    pub amendment_blocked: Option<bool>,
    /// The version number of the running rippled server
    pub build_version: Cow<'a, str>,
    /// Information about the most recently closed ledger that has not been
    /// validated
    pub closed_ledger: Option<LedgerInfo<'a>>,
    /// Range expression indicating the sequence numbers of the ledger versions
    /// in the database
    pub complete_ledgers: Cow<'a, str>,
    /// Performance metrics for RPC calls and JobQueue
    pub counters: Option<Value>,
    /// Items currently being run in the job queue
    pub current_activity: Option<Value>,
    /// Server hostname or RFC-1751 word based on node public key
    pub hostid: Option<Cow<'a, str>>,
    /// Amount of time spent waiting for I/O operations, in milliseconds
    pub io_latency_ms: u32,
    /// Number of times server had over 250 transactions waiting
    pub jq_trans_overflow: Option<Cow<'a, str>>,
    /// Information about the last ledger close
    pub last_close: LastClose,
    /// Detailed information about the current load state
    pub load: Option<Load<'a>>,
    /// Current transaction cost multiplier
    pub load_factor: u32,
    /// Transaction cost multiplier based on local load
    pub load_factor_local: Option<u32>,
    /// Transaction cost multiplier from network load
    pub load_factor_net: Option<u32>,
    /// Transaction cost multiplier from cluster load
    pub load_factor_cluster: Option<u32>,
    /// Transaction cost multiplier for open ledger
    pub load_factor_fee_escalation: Option<u32>,
    /// Transaction cost multiplier for queue
    pub load_factor_fee_queue: Option<u32>,
    /// Transaction cost multiplier excluding open ledger
    pub load_factor_server: Option<u32>,
    /// Network id for ledger
    pub network_id: Option<u32>,
    /// Number of connected peer servers
    pub peers: u32,
    /// List of ports listening for API commands
    pub ports: Option<Cow<'a, [Value]>>,
    /// Public key for peer-to-peer communications
    pub pubkey_node: Cow<'a, str>,
    /// Public key for ledger validations
    pub pubkey_validator: Option<Cow<'a, str>>,
    /// Reporting mode configuration information
    pub reporting: Option<Reporting<'a>>,
    /// Current server state
    pub server_state: Cow<'a, str>,
    /// Microseconds in current state
    pub server_state_duration_us: Option<Cow<'a, str>>,
    /// Server state accounting information
    pub state_accounting: Option<Value>,
    /// Current UTC time according to server
    pub time: Option<Cow<'a, str>>,
    /// Seconds server has been operational
    pub uptime: Option<u64>,
    /// Information about the most recent validated ledger
    pub validated_ledger: Option<LedgerInfo<'a>>,
    /// Minimum required trusted validations
    pub validation_quorum: u32,
    /// Validator list expiration time
    pub validator_list_expires: Option<u32>,
    /// Validator list information
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
pub struct Load<'a> {
    /// Information about job types and time spent
    pub job_types: Cow<'a, [Value]>,
    /// Number of threads in main job pool
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Reporting<'a> {
    /// List of P2P-mode servers
    pub etl_sources: Cow<'a, [Value]>,
    /// Whether server is writing to external database
    pub is_writer: bool,
    /// Last publish time
    pub last_publish_time: Cow<'a, str>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerInfo<'a> {
    /// Time since ledger close in seconds
    pub age: Option<u32>,
    /// Base fee in XRP (Not drops for some reason?)
    pub base_fee_xrp: Option<Amount<'a>>,
    /// Unique ledger hash
    pub hash: Cow<'a, str>,
    /// Minimum XRP reserve for accounts (Not drops for some reason?)
    pub reserve_base_xrp: Option<Amount<'a>>,
    /// Additional XRP reserve per owned object (Not drops for some reason?)
    pub reserve_inc_xrp: Option<Amount<'a>>,
    /// Ledger sequence number
    pub seq: u32,
}

/// Response format for the server_info command, which returns various
/// information about the rippled server's current state and configuration.
///
/// See Server Info:
/// `<https://xrpl.org/server_info.html#server_info>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ServerInfo<'a> {
    pub info: Info<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_deserialize() {
        let json = r#"{
            "info": {
                "build_version": "1.12.0",
                "complete_ledgers": "32570-82521761",
                "hostid": "LEST",
                "io_latency_ms": 1,
                "jq_trans_overflow": "0",
                "last_close": {
                    "converge_time_s": 3,
                    "proposers": 35
                },
                "load_factor": 1,
                "network_id": 10,
                "peers": 22,
                "ports": [
                    {
                        "port": "7777",
                        "protocol": ["ws"]
                    },
                    {
                        "port": "8080",
                        "protocol": ["ws"]
                    },
                    {
                        "port": "80",
                        "protocol": ["http"]
                    },
                    {
                        "port": "51235",
                        "protocol": ["peer"]
                    }
                ],
                "pubkey_node": "n9KQK8yvTDcZdGyhu2EGdDnFPEBSsY5wEGpU5GgpygTgLFsjQyPt",
                "server_state": "full",
                "server_state_duration_us": "91758491912",
                "time": "2023-Sep-13 22:12:31.377492 UTC",
                "uptime": 91948,
                "validated_ledger": {
                    "age": 1,
                    "base_fee_xrp": 0.00001,
                    "hash": "6872A6612DCEBCFC717FEBC66EB8CC2A4D5EEB2B0F15FC3DCD060049FCA47F31",
                    "reserve_base_xrp": 10,
                    "reserve_inc_xrp": 2,
                    "seq": 82521761
                },
                "validation_quorum": 28
            },
            "status": "success"
        }"#;

        let result: ServerInfo = serde_json::from_str(json).unwrap();

        assert_eq!(result.info.build_version, "1.12.0");
        assert_eq!(result.info.complete_ledgers, "32570-82521761");
        assert_eq!(result.info.hostid, Some("LEST".into()));
        assert_eq!(result.info.io_latency_ms, 1);
        assert_eq!(result.info.jq_trans_overflow, Some("0".into()));
        assert_eq!(result.info.last_close.converge_time_s, 3);
        assert_eq!(result.info.last_close.proposers, 35);
        assert_eq!(result.info.load_factor, 1);
        assert_eq!(result.info.network_id, Some(10));
        assert_eq!(result.info.peers, 22);
        assert_eq!(
            result.info.pubkey_node,
            "n9KQK8yvTDcZdGyhu2EGdDnFPEBSsY5wEGpU5GgpygTgLFsjQyPt"
        );
        assert_eq!(result.info.server_state, "full");
        assert_eq!(result.info.server_state_duration_us, Some("91758491912".into()));
        assert_eq!(
            result.info.time,
            Some("2023-Sep-13 22:12:31.377492 UTC".into())
        );
        assert_eq!(result.info.uptime, Some(91948));

        let validated_ledger = result.info.validated_ledger.unwrap();
        assert_eq!(validated_ledger.age, Some(1));
        assert_eq!(
            validated_ledger.base_fee_xrp,
            Some(Amount::XRPAmount(0.00001.into()))
        );
        assert_eq!(
            validated_ledger.hash,
            "6872A6612DCEBCFC717FEBC66EB8CC2A4D5EEB2B0F15FC3DCD060049FCA47F31"
        );
        assert_eq!(
            validated_ledger.reserve_base_xrp,
            Some(Amount::XRPAmount(10.into()))
        );
        assert_eq!(
            validated_ledger.reserve_inc_xrp,
            Some(Amount::XRPAmount(2.into()))
        );
        assert_eq!(validated_ledger.seq, 82521761);

        assert_eq!(result.info.validation_quorum, 28);
    }
}
