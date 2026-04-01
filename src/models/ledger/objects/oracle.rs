use crate::models::ledger::objects::LedgerEntryType;
use crate::models::transactions::PriceData;
use crate::models::{FlagCollection, Model, NoFlags};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CommonFields, LedgerObject};

/// The Oracle ledger entry holds data associated with a single price oracle object.
///
/// See Oracle:
/// `<https://xrpl.org/docs/references/protocol/ledger-data/ledger-entry-types/oracle>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Oracle<'a> {
    /// The base fields for all ledger object models.
    ///
    /// See Ledger Object Common Fields:
    /// `<https://xrpl.org/ledger-entry-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The XRPL account with update and delete privileges for the oracle.
    pub owner: Cow<'a, str>,
    /// An arbitrary value that identifies an oracle provider.
    pub provider: Cow<'a, str>,
    /// Describes the type of asset, such as "currency", "commodity", or "NFT".
    pub asset_class: Option<Cow<'a, str>>,
    /// An array of up to 10 PriceData objects, representing the price information.
    pub price_data_series: Option<Vec<PriceData>>,
    /// The time the data was last updated, represented in the ripple epoch.
    pub last_update_time: u32,
    /// An optional Universal Resource Identifier to reference price data off-chain.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
    /// A hint indicating which page of the owner directory links to this entry.
    pub owner_node: Option<Cow<'a, str>>,
    /// The identifying hash of the transaction that most recently modified this entry.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this entry.
    pub previous_txn_lgr_seq: u32,
}

impl Model for Oracle<'_> {}

impl<'a> LedgerObject<NoFlags> for Oracle<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

impl<'a> Oracle<'a> {
    pub fn new(
        index: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner: Cow<'a, str>,
        provider: Cow<'a, str>,
        asset_class: Option<Cow<'a, str>>,
        price_data_series: Option<Vec<PriceData>>,
        last_update_time: u32,
        uri: Option<Cow<'a, str>>,
        owner_node: Option<Cow<'a, str>>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Oracle,
                index,
                ledger_index,
            },
            owner,
            provider,
            asset_class,
            price_data_series,
            last_update_time,
            uri,
            owner_node,
            previous_txn_id,
            previous_txn_lgr_seq,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use crate::models::transactions::PriceData;
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let oracle = Oracle::new(
            Some(Cow::from("ForTest")),
            None,
            Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            Cow::from("chainlink"),
            Some(Cow::from("63757272656E6379")),
            Some(vec![PriceData {
                base_asset: Some("XRP".to_string()),
                quote_asset: Some("USD".to_string()),
                asset_price: Some("740".to_string()),
                scale: Some(1),
            }]),
            743609014,
            Some(Cow::from("https://example.com/oracle1")),
            Some(Cow::from("0")),
            Cow::from("ABC123DEF456"),
            12345678,
        );

        let serialized = serde_json::to_string(&oracle).unwrap();
        let deserialized: Oracle = serde_json::from_str(&serialized).unwrap();
        assert_eq!(oracle, deserialized);
    }

    #[test]
    fn test_new_minimal() {
        let oracle = Oracle::new(
            None,
            None,
            Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            Cow::from("provider1"),
            None,
            None,
            743609014,
            None,
            None,
            Cow::from("ABC123"),
            100,
        );

        assert_eq!(oracle.owner, "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW");
        assert_eq!(oracle.provider, "provider1");
        assert!(oracle.asset_class.is_none());
        assert!(oracle.price_data_series.is_none());
        assert_eq!(oracle.last_update_time, 743609014);
        assert!(oracle.uri.is_none());
        assert!(oracle.owner_node.is_none());
        assert_eq!(oracle.previous_txn_id, "ABC123");
        assert_eq!(oracle.previous_txn_lgr_seq, 100);
    }

    #[test]
    fn test_ledger_entry_type() {
        let oracle = Oracle::new(
            None,
            None,
            Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            Cow::from("provider1"),
            None,
            None,
            0,
            None,
            None,
            Cow::from("ABC123"),
            0,
        );

        assert_eq!(oracle.get_ledger_entry_type(), LedgerEntryType::Oracle);
    }

    #[test]
    fn test_with_multiple_price_data() {
        let oracle = Oracle::new(
            Some(Cow::from("TestIndex")),
            None,
            Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            Cow::from("chainlink"),
            Some(Cow::from("63757272656E6379")),
            Some(vec![
                PriceData {
                    base_asset: Some("XRP".to_string()),
                    quote_asset: Some("USD".to_string()),
                    asset_price: Some("740".to_string()),
                    scale: Some(1),
                },
                PriceData {
                    base_asset: Some("BTC".to_string()),
                    quote_asset: Some("USD".to_string()),
                    asset_price: Some("2600000".to_string()),
                    scale: Some(2),
                },
                PriceData {
                    base_asset: Some("ETH".to_string()),
                    quote_asset: Some("USD".to_string()),
                    asset_price: Some("160000".to_string()),
                    scale: Some(2),
                },
            ]),
            743609014,
            Some(Cow::from("https://example.com")),
            Some(Cow::from("0")),
            Cow::from("DEF789"),
            99999,
        );

        let series = oracle.price_data_series.as_ref().unwrap();
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].base_asset.as_deref(), Some("XRP"));
        assert_eq!(series[1].base_asset.as_deref(), Some("BTC"));
        assert_eq!(series[2].base_asset.as_deref(), Some("ETH"));
    }
}
