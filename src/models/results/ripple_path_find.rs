use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::path_find::PathStep;

/// Response from a ripple_path_find request, containing possible paths for
/// a payment.
///
/// See Ripple Path Find:
/// `<https://xrpl.org/ripple_path_find.html#ripple_path_find>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RipplePathFind<'a> {
    /// Array of objects with possible paths to take. If empty, then there are
    /// no paths connecting the source and destination accounts.
    pub alternatives: Cow<'a, [PathAlternative<'a>]>,
    /// Unique address of the account that would receive a payment transaction.
    pub destination_account: Cow<'a, str>,
    /// Array of currencies that the destination accepts, as 3-letter codes
    /// like "USD" or as 40-character hex like
    /// "015841551A748AD2C1F76FF6ECB0CCCD00000000".
    pub destination_currencies: Cow<'a, [Cow<'a, str>]>,
}

/// Represents a path from one possible source currency (held by the initiating
/// account) to the destination account and currency.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PathAlternative<'a> {
    /// Array of arrays of objects defining payment paths.
    pub paths_computed: Cow<'a, [Cow<'a, [PathStep<'a>]>]>,
    /// @deprecated Array of arrays of objects defining canonical payment
    /// paths. Should be disregarded if present.
    pub paths_canonical: Option<Cow<'a, [Cow<'a, [PathStep<'a>]>]>>,
    /// @deprecated Array of arrays of objects defining expanded payment
    /// paths. Should be disregarded if present.
    pub paths_expanded: Option<Cow<'a, [Cow<'a, [PathStep<'a>]>]>>,
    /// Currency Amount that the source would have to send along this path
    /// for the destination to receive the desired amount. Can be a string for
    /// XRP amounts or an object for issued currencies.
    pub source_amount: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ripple_path_find_deserialization() {
        let json = r#"{
            "alternatives": [
                {
                    "paths_canonical": [],
                    "paths_computed": [
                        [
                            {
                                "currency": "USD",
                                "issuer": "rpDMez6pm6dBve2TJsmDpv7Yae6V5Pyvy2",
                                "type": 48,
                                "type_hex": "0000000000000030"
                            },
                            {
                                "account": "rpDMez6pm6dBve2TJsmDpv7Yae6V5Pyvy2",
                                "type": 1,
                                "type_hex": "0000000000000001"
                            },
                            {
                                "account": "rfDeu7TPUmyvUrffexjMjq3mMcSQHZSYyA",
                                "type": 1,
                                "type_hex": "0000000000000001"
                            },
                            {
                                "account": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                                "type": 1,
                                "type_hex": "0000000000000001"
                            }
                        ]
                    ],
                    "source_amount": "207414"
                }
            ],
            "destination_account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "destination_currencies": [
                "USD",
                "JOE",
                "BTC",
                "DYM",
                "CNY",
                "EUR",
                "015841551A748AD2C1F76FF6ECB0CCCD00000000",
                "MXN",
                "XRP"
            ]
        }"#;

        let path_find: RipplePathFind = serde_json::from_str(json).unwrap();

        // Test basic fields
        assert_eq!(
            path_find.destination_account,
            "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59"
        );

        // Test destination currencies
        assert_eq!(path_find.destination_currencies.len(), 9);
        assert_eq!(path_find.destination_currencies[0], "USD");
        assert_eq!(path_find.destination_currencies[8], "XRP");

        // Test alternatives
        assert_eq!(path_find.alternatives.len(), 1);
        let alternative = &path_find.alternatives[0];

        // Test paths_computed
        assert_eq!(alternative.paths_computed.len(), 1);
        let path = &alternative.paths_computed[0];
        assert_eq!(path.len(), 4);

        // Test first path step
        let first_step = &path[0];
        assert_eq!(first_step.currency.as_deref(), Some("USD"));
        assert_eq!(
            first_step.issuer.as_deref(),
            Some("rpDMez6pm6dBve2TJsmDpv7Yae6V5Pyvy2")
        );
        assert_eq!(first_step.r#type, Some(48));

        // Test source amount
        assert_eq!(alternative.source_amount, "207414");

        // Test serialization roundtrip
        let serialized = serde_json::to_string(&path_find).unwrap();
        let deserialized: RipplePathFind = serde_json::from_str(&serialized).unwrap();
        assert_eq!(path_find, deserialized);
    }
}
