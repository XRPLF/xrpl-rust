use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response format for the ledger_closed method, which returns the unique
/// identifiers of the most recently closed ledger.
///
/// See Ledger Closed:
/// `<https://xrpl.org/ledger_closed.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LedgerClosed<'a> {
    /// The unique Hash of this ledger version, in hexadecimal.
    pub ledger_hash: Cow<'a, str>,

    /// The ledger index of this ledger version.
    pub ledger_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_closed_deserialize() {
        let json = r#"{
            "ledger_hash": "17ACB57A0F73B5160713E81FE72B2AC9F6064541004E272BD09F257D57C30C02",
            "ledger_index": 6643099
        }"#;

        let result: LedgerClosed = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.ledger_hash,
            "17ACB57A0F73B5160713E81FE72B2AC9F6064541004E272BD09F257D57C30C02"
        );
        assert_eq!(result.ledger_index, 6643099);
    }

    #[test]
    fn test_ledger_closed_serialize() {
        let ledger_closed = LedgerClosed {
            ledger_hash: "17ACB57A0F73B5160713E81FE72B2AC9F6064541004E272BD09F257D57C30C02".into(),
            ledger_index: 6643099,
        };

        let serialized = serde_json::to_string(&ledger_closed).unwrap();
        let deserialized: LedgerClosed = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ledger_closed, deserialized);
    }
}
