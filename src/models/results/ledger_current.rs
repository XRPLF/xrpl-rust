use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// Response format for the ledger_current method, which returns the sequence number
/// of the current open ledger these servers are working on.
///
/// See Ledger Current:
/// `<https://xrpl.org/ledger_current.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerCurrent<'a> {
    /// The ledger index of this ledger version.
    /// Note: A ledger_hash field is not provided, because the hash of the current
    /// ledger is constantly changing along with its contents.
    pub ledger_current_index: u32,
    /// Keep the lifetime parameter consistent with other result types
    #[serde(skip)]
    phantom: PhantomData<&'a ()>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_current_deserialize() {
        let json = r#"{
            "ledger_current_index": 6643240
        }"#;

        let result: LedgerCurrent = serde_json::from_str(json).unwrap();

        assert_eq!(result.ledger_current_index, 6643240);
    }

    #[test]
    fn test_ledger_current_serialize() {
        let ledger_current = LedgerCurrent {
            ledger_current_index: 6643240,
            ..LedgerCurrent::default()
        };

        let serialized = serde_json::to_string(&ledger_current).unwrap();
        let deserialized: LedgerCurrent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ledger_current, deserialized);
    }
}
