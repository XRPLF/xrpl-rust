use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::amount::XRPAmount;

/// Response format for the fee method, which reports the current state of
/// the open ledger requirements for the transaction cost.
///
/// See Fee:
/// `<https://xrpl.org/fee.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Fee<'a> {
    /// Number of transactions provisionally included in the in-progress
    /// ledger.
    pub current_ledger_size: Cow<'a, str>,
    /// Number of transactions currently queued for the next ledger.
    pub current_queue_size: Cow<'a, str>,
    /// Various information about the transaction cost (the Fee field of a
    /// transaction), in drops of XRP.
    pub drops: Drops<'a>,
    /// The approximate number of transactions expected to be included in the
    /// current ledger.
    pub expected_ledger_size: Cow<'a, str>,
    /// The Ledger Index of the current open ledger these stats describe.
    pub ledger_current_index: u32,
    /// Various information about the transaction cost, in fee levels.
    pub levels: Levels<'a>,
    /// The maximum number of transactions that the transaction queue can
    /// currently hold.
    pub max_queue_size: Cow<'a, str>,
}

/// Information about transaction costs in drops of XRP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Drops<'a> {
    /// The transaction cost required for a reference transaction to be
    /// included in a ledger under minimum load, represented in drops of XRP.
    pub base_fee: XRPAmount<'a>,
    /// An approximation of the median transaction cost among transactions
    /// included in the previous validated ledger, represented in drops of XRP.
    pub median_fee: XRPAmount<'a>,
    /// The minimum transaction cost for a reference transaction to be queued
    /// for a later ledger, represented in drops of XRP.
    pub minimum_fee: XRPAmount<'a>,
    /// The minimum transaction cost that a reference transaction must pay to
    /// be included in the current open ledger, represented in drops of XRP.
    pub open_ledger_fee: XRPAmount<'a>,
}

/// Information about transaction costs in fee levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Levels<'a> {
    /// The median transaction cost among transactions in the previous
    /// validated ledger, represented in fee levels.
    pub median_level: Cow<'a, str>,
    /// The minimum transaction cost required to be queued for a future
    /// ledger, represented in fee levels.
    pub minimum_level: Cow<'a, str>,
    /// The minimum transaction cost required to be included in the current
    /// open ledger, represented in fee levels.
    pub open_ledger_level: Cow<'a, str>,
    /// The equivalent of the minimum transaction cost, represented in fee
    /// levels.
    pub reference_level: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_deserialize() {
        let json = r#"{
            "current_ledger_size": "14",
            "current_queue_size": "0",
            "drops": {
                "base_fee": "10",
                "median_fee": "11000",
                "minimum_fee": "10",
                "open_ledger_fee": "10"
            },
            "expected_ledger_size": "24",
            "ledger_current_index": 26575101,
            "levels": {
                "median_level": "281600",
                "minimum_level": "256",
                "open_ledger_level": "256",
                "reference_level": "256"
            },
            "max_queue_size": "480"
        }"#;

        let fee: Fee = serde_json::from_str(json).unwrap();

        // Test top-level fields
        assert_eq!(fee.current_ledger_size, "14");
        assert_eq!(fee.current_queue_size, "0");
        assert_eq!(fee.expected_ledger_size, "24");
        assert_eq!(fee.ledger_current_index, 26575101);
        assert_eq!(fee.max_queue_size, "480");

        // Test drops
        assert_eq!(fee.drops.base_fee, XRPAmount::from("10"));
        assert_eq!(fee.drops.median_fee, XRPAmount::from("11000"));
        assert_eq!(fee.drops.minimum_fee, XRPAmount::from("10"));
        assert_eq!(fee.drops.open_ledger_fee, XRPAmount::from("10"));

        // Test levels
        assert_eq!(fee.levels.median_level, "281600");
        assert_eq!(fee.levels.minimum_level, "256");
        assert_eq!(fee.levels.open_ledger_level, "256");
        assert_eq!(fee.levels.reference_level, "256");
    }

    #[test]
    fn test_fee_serialize() {
        let fee = Fee {
            current_ledger_size: "14".into(),
            current_queue_size: "0".into(),
            drops: Drops {
                base_fee: XRPAmount::from("10"),
                median_fee: XRPAmount::from("11000"),
                minimum_fee: XRPAmount::from("10"),
                open_ledger_fee: XRPAmount::from("10"),
            },
            expected_ledger_size: "24".into(),
            ledger_current_index: 26575101,
            levels: Levels {
                median_level: "281600".into(),
                minimum_level: "256".into(),
                open_ledger_level: "256".into(),
                reference_level: "256".into(),
            },
            max_queue_size: "480".into(),
        };

        let serialized = serde_json::to_string(&fee).unwrap();
        let deserialized: Fee = serde_json::from_str(&serialized).unwrap();

        assert_eq!(fee, deserialized);
    }
}
