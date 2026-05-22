use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::Model;

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Marker, Request, RequestMethod};

/// The `nft_history` method retreives a list of transactions that involved the
/// specified NFToken.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NFTHistory<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// The unique identifier of an NFToken.
    /// The request returns past transactions of this NFToken.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let req = NFTHistory::new(
            Some("nh-1".into()),
            "00080000B4F4AFC5FBCBD76873F18006173D2193467D3EE70000099B00000000".into(),
            None,
            None,
            Some(1),
            Some(5000),
            Some(false),
            Some(true),
            Some(50),
            None,
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: NFTHistory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"nft_history\""));
    }
}
