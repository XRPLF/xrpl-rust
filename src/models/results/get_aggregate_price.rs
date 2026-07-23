use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Price statistics computed over the full set of oracle prices.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PriceStatistics<'a> {
    /// The simple mean of the collected prices.
    pub mean: Cow<'a, str>,
    /// The number of data points used to compute the mean.
    pub size: u32,
    /// The standard deviation of the collected prices.
    pub standard_deviation: Cow<'a, str>,
}

/// Response format for the `get_aggregate_price` method.
///
/// See Get Aggregate Price:
/// `<https://xrpl.org/get_aggregate_price.html>`
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GetAggregatePrice<'a> {
    /// Statistics from the full set of collected oracle prices.
    pub entire_set: PriceStatistics<'a>,
    /// Statistics from the trimmed set. Only present when `trim` was specified
    /// in the request.
    pub trimmed_set: Option<PriceStatistics<'a>>,
    /// The median of the collected oracle prices.
    pub median: Cow<'a, str>,
    /// The most recent `LastUpdateTime` across all included oracles (Ripple
    /// epoch seconds).
    pub time: u32,
    /// The ledger index of the current in-progress ledger used to generate
    /// this response.
    pub ledger_current_index: u32,
    /// If true, the information comes from a validated ledger version.
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_without_trim() {
        let json = r#"{
            "entire_set": {
                "mean": "0.75",
                "size": 3,
                "standard_deviation": "0.05"
            },
            "median": "0.74",
            "time": 743609014,
            "ledger_current_index": 4200000
        }"#;
        let result: GetAggregatePrice = serde_json::from_str(json).unwrap();
        assert_eq!(result.entire_set.mean, "0.75");
        assert_eq!(result.entire_set.size, 3);
        assert_eq!(result.median, "0.74");
        assert_eq!(result.time, 743609014);
        assert_eq!(result.ledger_current_index, 4200000);
        assert!(result.trimmed_set.is_none());
        assert!(result.validated.is_none());
    }

    #[test]
    fn test_deserialize_with_trim() {
        let json = r#"{
            "entire_set": {
                "mean": "0.75",
                "size": 10,
                "standard_deviation": "0.08"
            },
            "trimmed_set": {
                "mean": "0.74",
                "size": 6,
                "standard_deviation": "0.02"
            },
            "median": "0.735",
            "time": 743609200,
            "ledger_current_index": 4200100,
            "validated": false
        }"#;
        let result: GetAggregatePrice = serde_json::from_str(json).unwrap();
        let trimmed = result.trimmed_set.unwrap();
        assert_eq!(trimmed.mean, "0.74");
        assert_eq!(trimmed.size, 6);
        assert_eq!(result.validated, Some(false));
    }

    #[test]
    fn test_round_trip() {
        let result = GetAggregatePrice {
            entire_set: PriceStatistics {
                mean: "1.0".into(),
                size: 2,
                standard_deviation: "0.0".into(),
            },
            trimmed_set: None,
            median: "1.0".into(),
            time: 100,
            ledger_current_index: 1,
            validated: Some(true),
        };
        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: GetAggregatePrice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(result, deserialized);
    }
}
