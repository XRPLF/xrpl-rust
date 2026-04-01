use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::{Memo, PriceData, Signer, Transaction, TransactionType};
use crate::models::{FlagCollection, Model, NoFlags, XRPLModelResult};

use super::{CommonFields, CommonTransactionBuilder};

/// An OracleSet transaction creates or updates an Oracle ledger entry.
///
/// See OracleSet:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/oracleset>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OracleSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// A unique identifier of the price oracle for the account.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: Option<u32>,
    /// An arbitrary value that identifies an oracle provider, such as
    /// Chainlink, Band, or DIA. This field is a string, up to 256 ASCII
    /// hex encoded characters (128 bytes).
    pub provider: Option<Cow<'a, str>>,
    /// An optional Universal Resource Identifier to reference price data
    /// off-chain. This field is limited to 256 bytes.
    #[serde(rename = "URI")]
    pub uri: Option<Cow<'a, str>>,
    /// Describes the type of asset, such as "currency", "commodity", or
    /// "NFT". This field is a string, up to 16 ASCII hex encoded characters
    /// (8 bytes).
    pub asset_class: Option<Cow<'a, str>>,
    /// The time the data was last updated, represented in the ripple epoch.
    pub last_update_time: Option<u32>,
    /// An array of up to 10 PriceData objects, each representing one
    /// price data entry.
    pub price_data_series: Option<Vec<PriceData>>,
}

impl Model for OracleSet<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for OracleSet<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for OracleSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> OracleSet<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        oracle_document_id: Option<u32>,
        provider: Option<Cow<'a, str>>,
        uri: Option<Cow<'a, str>>,
        asset_class: Option<Cow<'a, str>>,
        last_update_time: Option<u32>,
        price_data_series: Option<Vec<PriceData>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::OracleSet,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            oracle_document_id,
            provider,
            uri,
            asset_class,
            last_update_time,
            price_data_series,
        }
    }

    /// Set the oracle document ID
    pub fn with_oracle_document_id(mut self, id: u32) -> Self {
        self.oracle_document_id = Some(id);
        self
    }

    /// Set the provider
    pub fn with_provider(mut self, provider: Cow<'a, str>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the URI
    pub fn with_uri(mut self, uri: Cow<'a, str>) -> Self {
        self.uri = Some(uri);
        self
    }

    /// Set the asset class
    pub fn with_asset_class(mut self, asset_class: Cow<'a, str>) -> Self {
        self.asset_class = Some(asset_class);
        self
    }

    /// Set the last update time
    pub fn with_last_update_time(mut self, time: u32) -> Self {
        self.last_update_time = Some(time);
        self
    }

    /// Set the price data series
    pub fn with_price_data_series(mut self, series: Vec<PriceData>) -> Self {
        self.price_data_series = Some(series);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_serde() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                fee: Some("12".into()),
                sequence: Some(391),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            oracle_document_id: Some(1),
            provider: Some("chainlink".into()),
            uri: Some("https://example.com/oracle1".into()),
            asset_class: Some("63757272656E6379".into()),
            last_update_time: Some(743609014),
            price_data_series: Some(vec![PriceData {
                base_asset: Some("XRP".to_string()),
                quote_asset: Some("USD".to_string()),
                asset_price: Some("740".to_string()),
                scale: Some(1),
            }]),
        };

        let serialized = serde_json::to_string(&oracle_set).unwrap();
        let deserialized: OracleSet = serde_json::from_str(&serialized).unwrap();
        assert_eq!(oracle_set, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_oracle_document_id(1)
        .with_provider("chainlink".into())
        .with_uri("https://example.com".into())
        .with_asset_class("63757272656E6379".into())
        .with_last_update_time(743609014)
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(596447)
        .with_source_tag(42);

        assert_eq!(oracle_set.oracle_document_id, Some(1));
        assert_eq!(oracle_set.provider.as_deref(), Some("chainlink"));
        assert_eq!(oracle_set.uri.as_deref(), Some("https://example.com"));
        assert_eq!(oracle_set.asset_class.as_deref(), Some("63757272656E6379"));
        assert_eq!(oracle_set.last_update_time, Some(743609014));
        assert_eq!(oracle_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(oracle_set.common_fields.sequence, Some(100));
        assert_eq!(oracle_set.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(oracle_set.common_fields.source_tag, Some(42));
    }

    #[test]
    fn test_default() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            oracle_set.common_fields.account,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert!(oracle_set.oracle_document_id.is_none());
        assert!(oracle_set.provider.is_none());
        assert!(oracle_set.uri.is_none());
        assert!(oracle_set.asset_class.is_none());
        assert!(oracle_set.last_update_time.is_none());
        assert!(oracle_set.price_data_series.is_none());
    }

    #[test]
    fn test_with_price_data() {
        let price_data = vec![
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
        ];

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(price_data.clone());

        let series = oracle_set.price_data_series.as_ref().unwrap();
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].base_asset.as_deref(), Some("XRP"));
        assert_eq!(series[0].quote_asset.as_deref(), Some("USD"));
        assert_eq!(series[0].asset_price.as_deref(), Some("740"));
        assert_eq!(series[0].scale, Some(1));
        assert_eq!(series[1].base_asset.as_deref(), Some("BTC"));
    }

    #[test]
    fn test_minimal() {
        let oracle_set = OracleSet::new(
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            oracle_set.common_fields.account,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert_eq!(oracle_set.oracle_document_id, Some(1));
    }

    #[test]
    fn test_new_constructor() {
        let price_data = vec![PriceData {
            base_asset: Some("XRP".to_string()),
            quote_asset: Some("USD".to_string()),
            asset_price: Some("740".to_string()),
            scale: Some(1),
        }];

        let oracle_set = OracleSet::new(
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            None,
            Some("12".into()),
            Some(596447),
            None,
            Some(391),
            None,
            None,
            None,
            Some(1),
            Some("chainlink".into()),
            Some("https://example.com/oracle1".into()),
            Some("63757272656E6379".into()),
            Some(743609014),
            Some(price_data),
        );

        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert_eq!(oracle_set.common_fields.fee, Some("12".into()));
        assert_eq!(oracle_set.common_fields.sequence, Some(391));
        assert_eq!(oracle_set.oracle_document_id, Some(1));
        assert_eq!(oracle_set.provider.as_deref(), Some("chainlink"));
        assert_eq!(oracle_set.last_update_time, Some(743609014));
        assert_eq!(oracle_set.price_data_series.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_transaction_type() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            *oracle_set.get_transaction_type(),
            TransactionType::OracleSet
        );
    }

    #[test]
    fn test_with_memos() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_oracle_document_id(1)
        .with_memo(Memo {
            memo_data: Some("oracle update".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(oracle_set.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_empty_price_data_series() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![]);

        assert_eq!(oracle_set.price_data_series.as_ref().unwrap().len(), 0);
    }

    #[test]
    fn test_price_data_partial_fields() {
        let price_data = PriceData {
            base_asset: Some("XRP".to_string()),
            quote_asset: None,
            asset_price: None,
            scale: None,
        };

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![price_data]);

        let series = oracle_set.price_data_series.as_ref().unwrap();
        assert_eq!(series[0].base_asset.as_deref(), Some("XRP"));
        assert!(series[0].quote_asset.is_none());
        assert!(series[0].asset_price.is_none());
        assert!(series[0].scale.is_none());
    }
}
