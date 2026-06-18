use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::Model;

use super::{CommonFields, Request};

/// Identifies a single Oracle object to include in the aggregate price
/// calculation.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct OracleDescriptor<'a> {
    /// The XRPL account that controls the Oracle object.
    pub account: Cow<'a, str>,
    /// The unique identifier of the price oracle for the account.
    pub oracle_document_id: u32,
}

/// The `get_aggregate_price` method retrieves the aggregate price of specified
/// Oracle objects, returning three price statistics: mean, median, and trimmed
/// mean.
///
/// See Get Aggregate Price:
/// `<https://xrpl.org/get_aggregate_price.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct GetAggregatePrice<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The currency code of the asset to be priced.
    pub base_asset: Cow<'a, str>,
    /// The currency code of the asset used to quote the base asset price.
    pub quote_asset: Cow<'a, str>,
    /// The oracle identifiers to include in the aggregate price calculation.
    pub oracles: Vec<OracleDescriptor<'a>>,
    /// Percentage of outliers to trim (1–25). When set, trimmed-mean statistics
    /// are returned in addition to the full-set statistics.
    pub trim: Option<u8>,
    /// Time range in seconds for filtering out older price data. Default 0
    /// (no filtering).
    pub trim_threshold: Option<u32>,
}

impl Model for GetAggregatePrice<'_> {}

impl<'a> Request<'a> for GetAggregatePrice<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> GetAggregatePrice<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        base_asset: Cow<'a, str>,
        quote_asset: Cow<'a, str>,
        oracles: Vec<OracleDescriptor<'a>>,
        trim: Option<u8>,
        trim_threshold: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: super::RequestMethod::GetAggregatePrice,
                id,
            },
            base_asset,
            quote_asset,
            oracles,
            trim,
            trim_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let req = GetAggregatePrice::new(
            None,
            "XRP".into(),
            "USD".into(),
            vec![OracleDescriptor {
                account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                oracle_document_id: 1,
            }],
            None,
            None,
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: GetAggregatePrice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"get_aggregate_price\""));
        assert!(serialized.contains("\"base_asset\":\"XRP\""));
    }

    #[test]
    fn test_with_trim() {
        let req = GetAggregatePrice::new(
            Some("test-1".into()),
            "BTC".into(),
            "USD".into(),
            vec![
                OracleDescriptor {
                    account: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into(),
                    oracle_document_id: 1,
                },
                OracleDescriptor {
                    account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                    oracle_document_id: 2,
                },
            ],
            Some(20),
            Some(60),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("\"trim\":20"));
        assert!(serialized.contains("\"trim_threshold\":60"));
        let deserialized: GetAggregatePrice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }
}
