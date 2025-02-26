use alloc::{borrow::Cow, vec::Vec};
use objects::LedgerEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod objects;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum LedgerVersionMap<'a> {
    Default(Ledger<'a>),
    V1(LedgerV1<'a>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ledger<'a> {
    #[serde(flatten)]
    pub base: BaseLedger<'a>,
    pub ledger_index: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LedgerV1<'a> {
    #[serde(flatten)]
    pub base: BaseLedger<'a>,
    pub ledger_index: Cow<'a, str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BaseLedger<'a> {
    pub account_hash: Cow<'a, str>,
    pub account_state: Option<Vec<LedgerEntry<'a>>>,
    pub close_flags: u32,
    pub close_time: u64,
    pub close_time_human: Cow<'a, str>,
    pub close_time_resolution: u32,
    pub close_time_iso: Cow<'a, str>,
    pub closed: bool,
    pub ledger_hash: Cow<'a, str>,
    pub parent_close_time: u64,
    pub parent_hash: Cow<'a, str>,
    pub total_coins: Cow<'a, str>,
    pub transaction_hash: Cow<'a, str>,
    pub transactions: Option<Vec<TransactionWithMetadata<'a>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TransactionWithMetadata<'a> {
    pub hash: Cow<'a, str>,
    pub metadata: Option<Value>, // TODO: Replace with actual metadata as soon as it's implemented
}
