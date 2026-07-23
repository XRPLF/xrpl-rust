use crate::models::ledger::objects::LedgerEntryType;
use crate::models::transactions::PriceData;
use crate::models::{Model, NoFlags};
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
    pub asset_class: Cow<'a, str>,
    /// An array of up to 10 PriceData objects, representing the price information.
    pub price_data_series: Vec<PriceData>,
    /// The time the data was last updated, represented in the ripple epoch.
    pub last_update_time: u32,
    /// An optional Universal Resource Identifier to reference price data off-chain.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
    /// A hint indicating which page of the owner directory links to this entry.
    pub owner_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this entry.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this entry.
    pub previous_txn_lgr_seq: u32,
    /// A unique identifier of the price oracle for the account, if present in
    /// the ledger entry returned by the server.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: Option<u32>,
}

impl Model for Oracle<'_> {}

impl<'a> LedgerObject<NoFlags> for Oracle<'a> {
    fn get_ledger_entry_type(&self) -> LedgerEntryType {
        self.common_fields.get_ledger_entry_type()
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use crate::models::transactions::PriceData;
    use crate::models::FlagCollection;
    use alloc::borrow::Cow;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serialize() {
        let oracle = Oracle {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Oracle,
                index: Some(Cow::from("ForTest")),
                ledger_index: None,
            },
            owner: Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            provider: Cow::from("636861696E6C696E6B"),
            asset_class: Cow::from("63757272656E6379"),
            price_data_series: vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("2E4".to_string()),
                scale: Some(1),
            }],
            last_update_time: 743609014,
            uri: Some(Cow::from("68747470733A2F2F6578616D706C652E636F6D")),
            owner_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from("ABC123DEF456"),
            previous_txn_lgr_seq: 12345678,
            oracle_document_id: Some(1),
        };

        let serialized = serde_json::to_string(&oracle).unwrap();
        let deserialized: Oracle = serde_json::from_str(&serialized).unwrap();
        assert_eq!(oracle, deserialized);
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "LedgerEntryType": "Oracle",
            "Flags": 0,
            "Owner": "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW",
            "Provider": "636861696E6C696E6B",
            "AssetClass": "63757272656E6379",
            "PriceDataSeries": [
                {
                    "PriceData": {
                        "BaseAsset": "XRP",
                        "QuoteAsset": "USD",
                        "AssetPrice": "2E4",
                        "Scale": 1
                    }
                }
            ],
            "LastUpdateTime": 743609014,
            "URI": "68747470733A2F2F6578616D706C652E636F6D",
            "OwnerNode": "0000000000000000",
            "PreviousTxnID": "ABC123DEF456",
            "PreviousTxnLgrSeq": 12345678,
            "OracleDocumentID": 1
        }"#;

        let oracle: Oracle = serde_json::from_str(json).unwrap();
        assert_eq!(oracle.owner, "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW");
        assert_eq!(oracle.provider, "636861696E6C696E6B");
        assert_eq!(oracle.asset_class, "63757272656E6379");
        assert_eq!(oracle.price_data_series.len(), 1);
        assert_eq!(oracle.price_data_series[0].base_asset, "XRP");
        assert_eq!(oracle.price_data_series[0].quote_asset, "USD");
        assert_eq!(oracle.price_data_series[0].asset_price, Some("2E4".into()));
        assert_eq!(oracle.price_data_series[0].scale, Some(1));
        assert_eq!(oracle.last_update_time, 743609014);
        assert_eq!(
            oracle.uri,
            Some("68747470733A2F2F6578616D706C652E636F6D".into())
        );
        assert_eq!(oracle.owner_node, "0000000000000000");
        assert_eq!(oracle.previous_txn_id, "ABC123DEF456");
        assert_eq!(oracle.previous_txn_lgr_seq, 12345678);
        assert_eq!(oracle.oracle_document_id, Some(1));
    }

    #[test]
    fn test_new_minimal() {
        let oracle = Oracle {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Oracle,
                index: None,
                ledger_index: None,
            },
            owner: Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            provider: Cow::from("70726F766964657231"),
            asset_class: Cow::from("63757272656E6379"),
            price_data_series: vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("2E4".to_string()),
                scale: Some(1),
            }],
            last_update_time: 743609014,
            uri: None,
            owner_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from("ABC123"),
            previous_txn_lgr_seq: 100,
            oracle_document_id: None,
        };

        assert_eq!(oracle.owner, "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW");
        assert_eq!(oracle.provider, "70726F766964657231");
        assert_eq!(oracle.asset_class, "63757272656E6379");
        assert_eq!(oracle.price_data_series.len(), 1);
        assert_eq!(oracle.last_update_time, 743609014);
        assert!(oracle.uri.is_none());
        assert_eq!(oracle.owner_node, "0000000000000000");
        assert_eq!(oracle.previous_txn_id, "ABC123");
        assert_eq!(oracle.previous_txn_lgr_seq, 100);
    }

    #[test]
    fn test_ledger_entry_type() {
        let oracle = Oracle {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Oracle,
                index: None,
                ledger_index: None,
            },
            owner: Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            provider: Cow::from("70726F766964657231"),
            asset_class: Cow::from("63757272656E6379"),
            price_data_series: vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("2E4".to_string()),
                scale: Some(1),
            }],
            last_update_time: 0,
            uri: None,
            owner_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from("ABC123"),
            previous_txn_lgr_seq: 0,
            oracle_document_id: None,
        };

        assert_eq!(oracle.get_ledger_entry_type(), LedgerEntryType::Oracle);
    }

    #[test]
    fn test_with_multiple_price_data() {
        let oracle = Oracle {
            common_fields: CommonFields {
                flags: FlagCollection::default(),
                ledger_entry_type: LedgerEntryType::Oracle,
                index: Some(Cow::from("TestIndex")),
                ledger_index: None,
            },
            owner: Cow::from("rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"),
            provider: Cow::from("636861696E6C696E6B"),
            asset_class: Cow::from("63757272656E6379"),
            price_data_series: vec![
                PriceData {
                    base_asset: "XRP".to_string(),
                    quote_asset: "USD".to_string(),
                    asset_price: Some("2E4".to_string()),
                    scale: Some(1),
                },
                PriceData {
                    base_asset: "BTC".to_string(),
                    quote_asset: "USD".to_string(),
                    asset_price: Some("27AC40".to_string()),
                    scale: Some(2),
                },
                PriceData {
                    base_asset: "ETH".to_string(),
                    quote_asset: "USD".to_string(),
                    asset_price: Some("27100".to_string()),
                    scale: Some(2),
                },
            ],
            last_update_time: 743609014,
            uri: Some(Cow::from("68747470733A2F2F6578616D706C652E636F6D")),
            owner_node: Cow::from("0000000000000000"),
            previous_txn_id: Cow::from("DEF789"),
            previous_txn_lgr_seq: 99999,
            oracle_document_id: None,
        };

        let series = oracle.price_data_series;
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].base_asset, "XRP");
        assert_eq!(series[1].base_asset, "BTC");
        assert_eq!(series[2].base_asset, "ETH");
    }
}
