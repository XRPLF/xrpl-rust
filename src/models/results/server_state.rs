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
    pub build_version: Cow<'a, str>,
    pub complete_ledgers: Cow<'a, str>,
    pub io_latency_ms: u32,
    pub jq_trans_overflow: JqTransOverflow<'a>,
    pub last_close: LastClose,
    pub load_base: u32,
    pub load_factor: u32,
    pub ports: Vec<PortDescriptor<'a>>,
    pub pubkey_node: Cow<'a, str>,
    pub pubkey_validator: Cow<'a, str>,
    pub server_state: Cow<'a, str>,
    pub server_state_duration_us: u32,
    pub state: State<'a>,
    pub state_accounting: RippledServerState<'a>,
    pub time: Cow<'a, str>,
    pub uptime: u32,
    pub validation_quorum: u32,
    pub amendment_blocked: Option<bool>,
    pub closed_ledger: Option<ValidatedLedger<'a>>,
    pub load: Option<Load<'a>>,
    pub load_factor_fee_escalation: Option<u32>,
    pub load_factor_fee_queue: Option<u32>,
    pub load_factor_fee_reference: Option<u32>,
    pub load_factor_server: Option<u32>,
    pub peers: Option<u32>,
    pub reporting: Option<Reporting<'a>>,
    pub validated_ledger: Option<ValidatedLedger<'a>>,
    pub validator_list_expires: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State<'a> {
    pub build_version: Cow<'a, str>,
    pub network_id: Option<u32>,
    pub validated_ledger: Option<ValidatedLedger<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JqTransOverflow<'a> {
    Int(u32),
    String(Cow<'a, str>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LastClose {
    pub converge_time: u32,
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
    pub etl_sources: Vec<ETLSource<'a>>,
    pub is_writer: bool,
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
    pub job_types: Vec<Cow<'a, str>>,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RippledServerState<'a> {
    pub disconnected: StateAccounting<'a>,
    pub connected: StateAccounting<'a>,
    pub syncing: StateAccounting<'a>,
    pub tracking: StateAccounting<'a>,
    pub full: StateAccounting<'a>,
    pub validating: StateAccounting<'a>,
    pub proposing: StateAccounting<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateAccounting<'a> {
    pub duration_us: Cow<'a, str>,
    pub transitions: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatedLedger<'a> {
    pub base_fee: XRPAmount<'a>,
    pub close_time: u32,
    pub hash: Cow<'a, str>,
    pub reserve_base: XRPAmount<'a>,
    pub reserve_inc: XRPAmount<'a>,
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
