use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::Amount;

/// Response from a path_find request, containing possible paths for a payment.
///
/// A successful result contains suggested paths and related information for making
/// a payment between accounts.
///
/// See Path Find:
/// `<https://xrpl.org/path_find.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathFind<'a> {
    /// Array of objects with suggested paths to take. If empty, then no paths were found
    /// connecting the source and destination accounts.
    pub alternatives: Cow<'a, [PathAlternative<'a>]>,
    /// Unique address of the account that would receive a transaction.
    pub destination_account: Cow<'a, str>,
    /// Currency Amount that the destination would receive in a transaction.
    pub destination_amount: Amount<'a>,
    /// Unique address that would send a transaction.
    pub source_account: Cow<'a, str>,
    /// If false, this is the result of an incomplete search. A later reply may have
    /// a better path. If true, then this is the best path found. Until you close the
    /// pathfinding request, rippled continues to send updates each time a new ledger closes.
    pub full_reply: Option<bool>,
}

/// Represents a path from one possible source currency (held by the initiating account)
/// to the destination account and currency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathAlternative<'a> {
    /// Array of arrays of objects defining payment paths.
    pub paths_computed: Cow<'a, [Cow<'a, [PathStep<'a>]>]>,
    /// Currency Amount that the source would have to send along this path for the
    /// destination to receive the desired amount.
    pub source_amount: Amount<'a>,
}

/// A PathStep represents an individual step along a Path.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone)]
pub struct PathStep<'a> {
    pub account: Option<Cow<'a, str>>,
    pub currency: Option<Cow<'a, str>>,
    pub issuer: Option<Cow<'a, str>>,
    pub r#type: Option<u8>,
    pub type_hex: Option<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use crate::models::Amount;

    use super::*;

    #[test]
    fn test_path_find_deserialization() {
        let json = r#"{
            "alternatives": [
                {
                    "paths_computed": [
                        [
                            {
                                "currency": "USD",
                                "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                                "type": 48,
                                "type_hex": "0000000000000030"
                            },
                            {
                                "account": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                                "type": 1,
                                "type_hex": "0000000000000001"
                            }
                        ]
                    ],
                    "source_amount": "251686"
                }
            ],
            "destination_account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "destination_amount": {
                "currency": "USD",
                "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                "value": "0.001"
            },
            "source_account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "full_reply": false
        }"#;

        let path_find: PathFind = serde_json::from_str(json).unwrap();

        // Test basic fields
        assert_eq!(
            path_find.destination_account,
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59"
        );
        assert_eq!(
            path_find.source_account,
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59"
        );
        assert_eq!(path_find.full_reply, Some(false));

        // Test alternatives
        assert_eq!(path_find.alternatives.len(), 1);
        let alternative = &path_find.alternatives[0];

        // Test paths_computed
        assert_eq!(alternative.paths_computed.len(), 1);
        let path = &alternative.paths_computed[0];
        assert_eq!(path.len(), 2);

        // Test first path step
        let first_step = &path[0];
        assert_eq!(first_step.currency.as_deref(), Some("USD"));
        assert_eq!(
            first_step.issuer.as_deref(),
            Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B")
        );
        assert_eq!(first_step.type_hex, Some("0000000000000030".into()));

        // Test second path step
        let second_step = &path[1];
        assert_eq!(
            second_step.account.as_deref(),
            Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B")
        );
        assert_eq!(second_step.type_hex, Some("0000000000000001".into()));

        // Test source_amount
        match &alternative.source_amount {
            Amount::XRPAmount(amount) => assert_eq!(amount.0, "251686"),
            _ => panic!("Expected XRPAmount"),
        }

        // Test destination_amount
        match &path_find.destination_amount {
            Amount::IssuedCurrencyAmount(amount) => {
                assert_eq!(amount.currency, "USD");
                assert_eq!(amount.issuer, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
                assert_eq!(amount.value, "0.001");
            }
            _ => panic!("Expected IssuedCurrencyAmount"),
        }

        // Test serialization
        let serialized = serde_json::to_string(&path_find).unwrap();
        let deserialized: PathFind = serde_json::from_str(&serialized).unwrap();
        assert_eq!(path_find, deserialized);
    }
}
