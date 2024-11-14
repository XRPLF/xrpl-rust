use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::Model;

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Marker, Request, RequestMethod};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NFTHistory<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    pub nft_id: Cow<'a, str>,
    pub ledger_index_min: Option<u32>,
    pub ledger_index_max: Option<u32>,
    pub binary: Option<bool>,
    pub forward: Option<bool>,
    pub limit: Option<u32>,
    pub marker: Option<Marker<'a>>,
}

impl Model for NFTHistory<'_> {}

impl<'a> Request<'a> for NFTHistory<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> NFTHistory<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        nft_id: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        ledger_index_min: Option<u32>,
        ledger_index_max: Option<u32>,
        binary: Option<bool>,
        forward: Option<bool>,
        limit: Option<u32>,
        marker: Option<Marker<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::NFTHistory,
                id,
            },
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            nft_id,
            ledger_index_min,
            ledger_index_max,
            binary,
            forward,
            limit,
            marker,
        }
    }
}
