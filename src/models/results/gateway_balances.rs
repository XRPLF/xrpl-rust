use alloc::{borrow::Cow, collections::BTreeMap};

use serde::{Deserialize, Serialize};

/// Response format for the gateway_balances method, which calculates the total
/// balances issued by a given account.
///
/// See Gateway Balances:
/// `<https://xrpl.org/gateway_balances.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GatewayBalances<'a> {
    /// The address of the account that issued the balances.
    pub account: Cow<'a, str>,
    /// (Omitted if empty) Total amounts held that are issued by others.
    /// In the recommended configuration, the issuing address should have none.
    pub assets: Option<BTreeMap<Cow<'a, str>, Cow<'a, [AssetBalance<'a>]>>>,
    /// (Omitted if empty) Amounts issued to the hotwallet addresses from the
    /// request. The keys are addresses and the values are arrays of currency
    /// amounts they hold.
    pub balances: Option<BTreeMap<Cow<'a, str>, Cow<'a, [AssetBalance<'a>]>>>,
    /// (Omitted if empty) Total amounts issued to addresses not excluded,
    /// as a map of currencies to the total value issued.
    pub obligations: Option<BTreeMap<Cow<'a, str>, Cow<'a, str>>>,
    /// (May be omitted) The identifying hash of the ledger version that was
    /// used to generate this response.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// (May be omitted) The ledger index of the ledger version that was used
    /// to generate this response.
    pub ledger_index: Option<u32>,
    /// (Omitted if ledger_current_index is provided) The ledger index of the
    /// current in-progress ledger version, which was used to retrieve this
    /// information.
    pub ledger_current_index: Option<u32>,
}

/// Represents a balance for a specific currency in the assets field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetBalance<'a> {
    /// The currency code of the balance.
    pub currency: Cow<'a, str>,
    /// The amount of the currency.
    pub value: Cow<'a, str>,
}

impl<'a> Default for AssetBalance<'a> {
    fn default() -> Self {
        Self {
            currency: Cow::Borrowed("XRP"),
            value: Cow::Borrowed("0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_balances_deserialize() {
        let json = r#"{
            "account": "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q",
            "assets": {
                "r9F6wk8HkXrgYWoJ7fsv4VrUBVoqDVtzkH": [
                    {
                        "currency": "BTC",
                        "value": "5444166510000000e-26"
                    }
                ],
                "rPFLkxQk6xUGdGYEykqe7PR25Gr7mLHDc8": [
                    {
                        "currency": "EUR",
                        "value": "4000000000000000e-27"
                    }
                ],
                "rPU6VbckqCLW4kb51CWqZdxvYyQrQVsnSj": [
                    {
                        "currency": "BTC",
                        "value": "1029900000000000e-26"
                    }
                ],
                "rpR95n1iFkTqpoy1e878f4Z1pVHVtWKMNQ": [
                    {
                        "currency": "BTC",
                        "value": "4000000000000000e-30"
                    }
                ],
                "rwmUaXsWtXU4Z843xSYwgt1is97bgY8yj6": [
                    {
                        "currency": "BTC",
                        "value": "8700000000000000e-30"
                    }
                ]
            },
            "balances": {
                "rKm4uWpg9tfwbVSeATv4KxDe6mpE9yPkgJ": [
                    {
                        "currency": "EUR",
                        "value": "29826.1965999999"
                    }
                ],
                "ra7JkEzrgeKHdzKgo4EUUVBnxggY4z37kt": [
                    {
                        "currency": "USD",
                        "value": "13857.70416"
                    }
                ]
            },
            "ledger_hash": "61DDBF304AF6E8101576BF161D447CA8E4F0170DDFBEAFFD993DC9383D443388",
            "ledger_index": 14483195,
            "obligations": {
                "BTC": "5908.324927635318",
                "EUR": "992471.7419793958",
                "GBP": "4991.38706013193",
                "USD": "1997134.20229482"
            },
            "validated": true
        }"#;

        let result: GatewayBalances = serde_json::from_str(json).unwrap();

        // Test basic fields
        assert_eq!(result.account, "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q");
        assert_eq!(
            result.ledger_hash.unwrap(),
            "61DDBF304AF6E8101576BF161D447CA8E4F0170DDFBEAFFD993DC9383D443388"
        );
        assert_eq!(result.ledger_index.unwrap(), 14483195);

        // Test assets
        let assets = result.assets.unwrap();
        assert_eq!(assets.len(), 5);
        let btc_asset = &assets["r9F6wk8HkXrgYWoJ7fsv4VrUBVoqDVtzkH"][0];
        assert_eq!(btc_asset.currency, "BTC");
        assert_eq!(btc_asset.value, "5444166510000000e-26");

        // Test balances
        let balances = result.balances.unwrap();
        assert_eq!(balances.len(), 2);
        let eur_balance = &balances["rKm4uWpg9tfwbVSeATv4KxDe6mpE9yPkgJ"][0];
        assert_eq!(eur_balance.currency, "EUR");
        assert_eq!(eur_balance.value, "29826.1965999999");

        // Test obligations
        let obligations = result.obligations.unwrap();
        assert_eq!(obligations.len(), 4);
        assert_eq!(obligations["BTC"], "5908.324927635318");
        assert_eq!(obligations["EUR"], "992471.7419793958");
        assert_eq!(obligations["GBP"], "4991.38706013193");
        assert_eq!(obligations["USD"], "1997134.20229482");
    }

    #[test]
    fn test_gateway_balances_serialize() {
        let mut assets = BTreeMap::new();
        assets.insert(
            "r9F6wk8HkXrgYWoJ7fsv4VrUBVoqDVtzkH".into(),
            alloc::vec![AssetBalance {
                currency: "BTC".into(),
                value: "5444166510000000e-26".into(),
            }]
            .into(),
        );

        let mut balances = BTreeMap::new();
        balances.insert(
            "rKm4uWpg9tfwbVSeATv4KxDe6mpE9yPkgJ".into(),
            alloc::vec![AssetBalance {
                currency: "EUR".into(),
                value: "29826.1965999999".into(),
            }]
            .into(),
        );

        let mut obligations = BTreeMap::new();
        obligations.insert("BTC".into(), "5908.324927635318".into());
        obligations.insert("EUR".into(), "992471.7419793958".into());

        let gateway_balances = GatewayBalances {
            account: "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q".into(),
            assets: Some(assets),
            balances: Some(balances),
            obligations: Some(obligations),
            ledger_hash: Some(
                "61DDBF304AF6E8101576BF161D447CA8E4F0170DDFBEAFFD993DC9383D443388".into(),
            ),
            ledger_index: Some(14483195),
            ledger_current_index: None,
        };

        let serialized = serde_json::to_string(&gateway_balances).unwrap();
        let deserialized: GatewayBalances = serde_json::from_str(&serialized).unwrap();

        assert_eq!(gateway_balances, deserialized);
    }
}
