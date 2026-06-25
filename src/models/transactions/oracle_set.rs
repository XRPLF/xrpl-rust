use alloc::borrow::Cow;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::{Memo, PriceData, Signer, Transaction, TransactionType};
use crate::models::{FlagCollection, Model, NoFlags, XRPLModelException, XRPLModelResult};

use super::{CommonFields, CommonTransactionBuilder};

/// Maximum number of PriceData entries allowed in a single OracleSet transaction.
/// Matches rippled `kMaxOracleDataSeries` in `Protocol.h`.
const MAX_ORACLE_DATA_SERIES: u32 = 10;
/// Maximum decoded byte length for the `Provider` Blob field.
/// The hex string on the wire may therefore be up to 512 characters long.
/// Matches rippled `kMaxOracleProvider = 256` in `Protocol.h`.
const MAX_ORACLE_PROVIDER_DECODED_BYTES: usize = 256;
/// Maximum decoded byte length for the `URI` Blob field.
/// Matches rippled `kMaxOracleUri = 256` in `Protocol.h`.
const MAX_ORACLE_URI_DECODED_BYTES: usize = 256;
/// Maximum decoded byte length for the `AssetClass` Blob field.
/// Matches rippled `kMaxOracleSymbolClass = 16` in `Protocol.h`.
const MAX_ORACLE_ASSET_CLASS_DECODED_BYTES: usize = 16;

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
    pub oracle_document_id: u32,
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
    pub last_update_time: u32,
    /// An array of 1 to 10 PriceData objects, each representing one
    /// price data entry.
    pub price_data_series: Vec<PriceData>,
}

impl Model for OracleSet<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        validate_optional_blob(
            "provider",
            self.provider.as_deref(),
            MAX_ORACLE_PROVIDER_DECODED_BYTES,
        )?;
        validate_optional_blob("uri", self.uri.as_deref(), MAX_ORACLE_URI_DECODED_BYTES)?;
        validate_optional_blob(
            "asset_class",
            self.asset_class.as_deref(),
            MAX_ORACLE_ASSET_CLASS_DECODED_BYTES,
        )?;

        let series = &self.price_data_series;
        if series.is_empty() {
            return Err(XRPLModelException::ValueTooLow {
                field: "price_data_series".into(),
                min: 1,
                found: 0,
            });
        }
        if series.len() as u32 > MAX_ORACLE_DATA_SERIES {
            return Err(XRPLModelException::ValueTooHigh {
                field: "price_data_series".into(),
                max: MAX_ORACLE_DATA_SERIES,
                found: series.len() as u32,
            });
        }

        let mut pairs = BTreeSet::new();
        for entry in series {
            entry.validate()?;
            if entry.base_asset == entry.quote_asset {
                return Err(XRPLModelException::ValueEqualsValue {
                    field1: "base_asset".into(),
                    field2: "quote_asset".into(),
                });
            }
            let pair = (entry.base_asset.clone(), entry.quote_asset.clone());
            if !pairs.insert(pair) {
                return Err(XRPLModelException::InvalidValue {
                    field: "price_data_series".into(),
                    expected: "unique BaseAsset/QuoteAsset pairs".into(),
                    found: alloc::format!("{}/{}", entry.base_asset, entry.quote_asset),
                });
            }
        }
        Ok(())
    }
}

fn validate_optional_blob(
    field: &'static str,
    value: Option<&str>,
    max_bytes: usize,
) -> XRPLModelResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let bytes = hex::decode(value).map_err(|e| {
        use hex::FromHexError;
        let reason = match e {
            FromHexError::OddLength => "hex string has odd length (incomplete byte)",
            FromHexError::InvalidHexCharacter { .. } => "non-hexadecimal character in string",
            FromHexError::InvalidStringLength => "invalid hex string length",
        };
        XRPLModelException::InvalidValue {
            field: field.into(),
            expected: alloc::format!("a valid hex-encoded Blob string ({reason})"),
            found: value.into(),
        }
    })?;
    // rippled `isInvalidLength` rejects empty blobs (length == 0) with
    // `temMALFORMED`, matching the binary-codec requirement that Blob fields
    // be non-empty when present.
    if bytes.is_empty() {
        return Err(XRPLModelException::InvalidValue {
            field: field.into(),
            expected: "a non-empty hex-encoded Blob string (empty strings are rejected)".into(),
            found: value.into(),
        });
    }
    if bytes.len() > max_bytes {
        return Err(XRPLModelException::ValueTooLong {
            field: field.into(),
            max: max_bytes,
            found: bytes.len(),
        });
    }
    Ok(())
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
        oracle_document_id: u32,
        provider: Option<Cow<'a, str>>,
        uri: Option<Cow<'a, str>>,
        asset_class: Option<Cow<'a, str>>,
        last_update_time: u32,
        price_data_series: Vec<PriceData>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::OracleSet,
                account_txn_id,
                fee,
                flags: FlagCollection::default(),
                last_ledger_sequence,
                memos,
                network_id: None,
                sequence,
                signers,
                signing_pub_key: None, // filled by the signing layer
                source_tag,
                ticket_sequence,
                txn_signature: None, // filled by the signing layer
            },
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
        self.oracle_document_id = id;
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
        self.last_update_time = time;
        self
    }

    /// Set the price data series
    pub fn with_price_data_series(mut self, series: Vec<PriceData>) -> Self {
        self.price_data_series = series;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    /// Canonical test account used across all OracleSet unit tests.
    const TEST_ACCOUNT: &str = "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW";
    const TEST_FEE: &str = "12";
    const TEST_SEQUENCE: u32 = 391;
    const TEST_LAST_LEDGER: u32 = 596447;
    const TEST_DOC_ID: u32 = 1;
    const TEST_LAST_UPDATE_TIME: u32 = 743609014;
    /// "chainlink" hex-encoded (Provider is a Blob field).
    const TEST_PROVIDER: &str = "636861696E6C696E6B";
    /// "currency" hex-encoded (AssetClass is a Blob field).
    const TEST_ASSET_CLASS: &str = "63757272656E6379";

    #[test]
    fn test_serde() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                fee: Some("12".into()),
                sequence: Some(391),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            oracle_document_id: 1,
            provider: Some("636861696E6C696E6B".into()),
            uri: Some("68747470733A2F2F6578616D706C652E636F6D2F6F7261636C6531".into()),
            asset_class: Some("63757272656E6379".into()),
            last_update_time: 743609014,
            price_data_series: vec![PriceData {
                base_asset: "EUR".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("740".to_string()), // hex: 1856 decimal,
                scale: Some(1),
            }],
        };

        let serialized = serde_json::to_string(&oracle_set)
            .expect("OracleSet should serialize to JSON without error");
        let deserialized: OracleSet = serde_json::from_str(&serialized)
            .expect("OracleSet should deserialize from its own JSON output");
        assert_eq!(oracle_set, deserialized);
        // `XRP` was rejected as a PriceData asset; ensure this model validates.
        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
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

        assert_eq!(oracle_set.oracle_document_id, 1);
        assert_eq!(oracle_set.provider.as_deref(), Some("chainlink"));
        assert_eq!(oracle_set.uri.as_deref(), Some("https://example.com"));
        assert_eq!(oracle_set.asset_class.as_deref(), Some("63757272656E6379"));
        assert_eq!(oracle_set.last_update_time, 743609014);
        assert_eq!(oracle_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(oracle_set.common_fields.sequence, Some(100));
        assert_eq!(oracle_set.common_fields.last_ledger_sequence, Some(596447));
        assert_eq!(oracle_set.common_fields.source_tag, Some(42));
    }

    #[test]
    fn test_default() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(oracle_set.common_fields.account, TEST_ACCOUNT);
        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert_eq!(oracle_set.oracle_document_id, 0);
        assert!(oracle_set.provider.is_none());
        assert!(oracle_set.uri.is_none());
        assert!(oracle_set.asset_class.is_none());
        assert_eq!(oracle_set.last_update_time, 0);
        assert!(oracle_set.price_data_series.is_empty());
    }

    #[test]
    fn test_with_price_data() {
        let price_data = vec![
            PriceData {
                base_asset: "EUR".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("740".to_string()), // hex: 1856 decimal,
                scale: Some(1),
            },
            PriceData {
                base_asset: "BTC".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("2600000".to_string()), // hex: 39845888 decimal,
                scale: Some(2),
            },
        ];

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(price_data.clone());

        let series = oracle_set.price_data_series;
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].base_asset, "EUR");
        assert_eq!(series[0].quote_asset, "USD");
        assert_eq!(series[0].asset_price.as_deref(), Some("740"));
        assert_eq!(series[0].scale, Some(1));
        assert_eq!(series[1].base_asset, "BTC");
    }

    #[test]
    fn test_minimal() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: TEST_DOC_ID,
            last_update_time: TEST_LAST_UPDATE_TIME,
            price_data_series: vec![],
            ..Default::default()
        };

        assert_eq!(oracle_set.common_fields.account, TEST_ACCOUNT);
        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert_eq!(oracle_set.oracle_document_id, TEST_DOC_ID);
    }

    #[test]
    fn test_new_constructor() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                fee: Some(TEST_FEE.into()),
                last_ledger_sequence: Some(TEST_LAST_LEDGER),
                sequence: Some(TEST_SEQUENCE),
                ..Default::default()
            },
            oracle_document_id: TEST_DOC_ID,
            // Non-hex plain string used here intentionally to test that the
            // constructor stores values verbatim (validation is in get_errors).
            provider: Some("chainlink".into()),
            uri: Some("68747470733A2F2F6578616D706C652E636F6D2F6F7261636C6531".into()),
            asset_class: Some(TEST_ASSET_CLASS.into()),
            last_update_time: TEST_LAST_UPDATE_TIME,
            price_data_series: vec![PriceData {
                base_asset: "EUR".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("2E4".to_string()), // hex: 740 decimal,
                scale: Some(1),
            }],
        };

        assert_eq!(
            oracle_set.common_fields.transaction_type,
            TransactionType::OracleSet
        );
        assert_eq!(oracle_set.common_fields.fee, Some(TEST_FEE.into()));
        assert_eq!(oracle_set.common_fields.sequence, Some(TEST_SEQUENCE));
        assert_eq!(oracle_set.oracle_document_id, TEST_DOC_ID);
        assert_eq!(oracle_set.provider.as_deref(), Some("chainlink"));
        assert_eq!(oracle_set.last_update_time, TEST_LAST_UPDATE_TIME);
        assert_eq!(oracle_set.price_data_series.len(), 1);
    }

    #[test]
    fn test_transaction_type() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
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
                account: TEST_ACCOUNT.into(),
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
    fn test_empty_price_data_series_rejected() {
        // When `price_data_series` is present, rippled requires at least 1 entry.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![]);

        let err = oracle_set.get_errors().unwrap_err();
        assert_eq!(
            err,
            XRPLModelException::ValueTooLow {
                field: "price_data_series".into(),
                min: 1,
                found: 0,
            }
        );
    }

    #[test]
    fn test_price_data_optional_update_fields() {
        // BaseAsset and QuoteAsset are required protocol fields. AssetPrice and
        // Scale remain optional; omitting AssetPrice on update deletes the pair.
        let price_data = PriceData {
            base_asset: "EUR".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: None,
            scale: None,
        };

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![price_data]);

        let series = oracle_set.price_data_series;
        assert_eq!(series[0].base_asset, "EUR");
        assert_eq!(series[0].quote_asset, "USD");
        assert!(series[0].asset_price.is_none());
        assert!(series[0].scale.is_none());
    }

    #[test]
    fn test_price_data_series_max_valid() {
        // Use valid 3-char ISO-style codes for the per-entry currency validation.
        let series: Vec<PriceData> = (0..10)
            .map(|i| PriceData {
                base_asset: alloc::format!("A{i:02}"),
                quote_asset: "USD".to_string(),
                asset_price: Some("100".to_string()), // hex: 256 decimal,
                scale: Some(1),
            })
            .collect();

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(series);

        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_price_data_series_exceeds_max() {
        let series: Vec<PriceData> = (0..11)
            .map(|i| PriceData {
                base_asset: alloc::format!("A{i:02}"),
                quote_asset: "USD".to_string(),
                asset_price: Some("100".to_string()), // hex: 256 decimal,
                scale: Some(1),
            })
            .collect();

        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(series);

        let err = oracle_set.get_errors().unwrap_err();
        assert_eq!(
            err,
            XRPLModelException::ValueTooHigh {
                field: "price_data_series".into(),
                max: 10,
                found: 11,
            }
        );
    }

    #[test]
    fn test_scale_too_high_rejected() {
        // Per rippled `kMaxPriceScale = 20` in Protocol.h; scale 21 is rejected.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "EUR".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(21),
        }]);

        let err = oracle_set.get_errors().unwrap_err();
        assert_eq!(
            err,
            XRPLModelException::ValueTooHigh {
                field: "scale".into(),
                max: 20,
                found: 21,
            }
        );
    }

    #[test]
    fn test_scale_at_max_ok() {
        // Boundary: scale = 20 is explicitly permitted (kMaxPriceScale = 20).
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "EUR".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(20),
        }]);

        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_scale_mid_range_ok() {
        // Values 11-20 must also pass; they were incorrectly rejected before.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "EUR".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(15),
        }]);

        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_asset_price_and_scale_must_be_paired() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "XRP".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: None,
        }]);

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { ref field, .. } if field == "price_data"
        ));
    }

    #[test]
    fn test_invalid_base_asset_rejected() {
        // A 4-character code is neither a valid ISO code nor a 40-char hex.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "EURO".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(1),
        }]);

        let err = oracle_set.get_errors().unwrap_err();
        assert!(matches!(
            err,
            XRPLModelException::InvalidValue { ref field, .. } if field == "base_asset"
        ));
    }

    #[test]
    fn test_xrp_as_asset_accepted() {
        // XRP is valid as an oracle currency code.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "XRP".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(1),
        }]);

        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_hex_currency_accepted() {
        // 40-character hex currency codes are valid.
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "0000000000000000000000005553440000000000".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(0),
        }]);

        assert!(oracle_set.get_errors().is_ok());
    }

    #[test]
    fn test_oracle_metadata_lengths_rejected() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            provider: Some("AA".repeat(MAX_ORACLE_PROVIDER_DECODED_BYTES + 1).into()),
            price_data_series: vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("100".to_string()), // hex: 256 decimal,
                scale: Some(1),
            }],
            ..Default::default()
        };

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::ValueTooLong { ref field, max, .. }
                if field == "provider" && max == MAX_ORACLE_PROVIDER_DECODED_BYTES
        ));
    }

    #[test]
    fn test_oracle_metadata_must_be_hex() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            provider: Some("chainlink".into()),
            price_data_series: vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("100".to_string()), // hex: 256 decimal,
                scale: Some(1),
            }],
            ..Default::default()
        };

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { ref field, .. } if field == "provider"
        ));
    }

    #[test]
    fn test_asset_price_full_u64_range_accepted() {
        // AssetPrice is a plain UInt64 — the full unsigned range is valid,
        // including 0x8000000000000000..=0xFFFFFFFFFFFFFFFF.
        // xrpl.js integration test uses "ffffffffffffffff" successfully.
        for price in ["8000000000000000", "FFFFFFFFFFFFFFFF", "1", "0"] {
            let oracle_set = OracleSet {
                common_fields: CommonFields {
                    account: TEST_ACCOUNT.into(),
                    transaction_type: TransactionType::OracleSet,
                    ..Default::default()
                },
                ..Default::default()
            }
            .with_price_data_series(vec![PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some(price.to_string()),
                scale: Some(1),
            }]);

            assert!(
                oracle_set.get_errors().is_ok(),
                "AssetPrice {price} should be valid"
            );
        }
    }

    #[test]
    fn test_asset_price_non_hex_rejected() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "XRP".to_string(),
            quote_asset: "USD".to_string(),
            asset_price: Some("ZZZZZZZZZZZZZZZZ".to_string()),
            scale: Some(1),
        }]);

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { ref field, .. } if field == "asset_price"
        ));
    }

    #[test]
    fn test_empty_blob_fields_rejected() {
        // rippled rejects zero-length Provider/URI/AssetClass with temMALFORMED.
        for (field_name, oracle) in [
            (
                "provider",
                OracleSet {
                    common_fields: CommonFields {
                        account: TEST_ACCOUNT.into(),
                        transaction_type: TransactionType::OracleSet,
                        ..Default::default()
                    },
                    provider: Some("".into()),
                    price_data_series: vec![PriceData {
                        base_asset: "XRP".to_string(),
                        quote_asset: "USD".to_string(),
                        asset_price: Some("100".to_string()), // hex: 256 decimal,
                        scale: Some(1),
                    }],
                    ..Default::default()
                },
            ),
            (
                "uri",
                OracleSet {
                    common_fields: CommonFields {
                        account: TEST_ACCOUNT.into(),
                        transaction_type: TransactionType::OracleSet,
                        ..Default::default()
                    },
                    uri: Some("".into()),
                    price_data_series: vec![PriceData {
                        base_asset: "XRP".to_string(),
                        quote_asset: "USD".to_string(),
                        asset_price: Some("100".to_string()), // hex: 256 decimal,
                        scale: Some(1),
                    }],
                    ..Default::default()
                },
            ),
            (
                "asset_class",
                OracleSet {
                    common_fields: CommonFields {
                        account: TEST_ACCOUNT.into(),
                        transaction_type: TransactionType::OracleSet,
                        ..Default::default()
                    },
                    asset_class: Some("".into()),
                    price_data_series: vec![PriceData {
                        base_asset: "XRP".to_string(),
                        quote_asset: "USD".to_string(),
                        asset_price: Some("100".to_string()), // hex: 256 decimal,
                        scale: Some(1),
                    }],
                    ..Default::default()
                },
            ),
        ] {
            assert!(
                matches!(
                    oracle.get_errors().unwrap_err(),
                    XRPLModelException::InvalidValue { ref field, .. } if field == field_name
                ),
                "empty {field_name} should be rejected"
            );
        }
    }

    #[test]
    fn test_duplicate_price_data_pair_rejected() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![
            PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("100".to_string()), // hex: 256 decimal,
                scale: Some(1),
            },
            PriceData {
                base_asset: "XRP".to_string(),
                quote_asset: "USD".to_string(),
                asset_price: Some("101".to_string()),
                scale: Some(1),
            },
        ]);

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::InvalidValue { ref field, .. } if field == "price_data_series"
        ));
    }

    #[test]
    fn test_same_base_quote_rejected() {
        let oracle_set = OracleSet {
            common_fields: CommonFields {
                account: TEST_ACCOUNT.into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_price_data_series(vec![PriceData {
            base_asset: "XRP".to_string(),
            quote_asset: "XRP".to_string(),
            asset_price: Some("100".to_string()), // hex: 256 decimal,
            scale: Some(1),
        }]);

        assert!(matches!(
            oracle_set.get_errors().unwrap_err(),
            XRPLModelException::ValueEqualsValue { ref field1, ref field2 }
                if field1 == "base_asset" && field2 == "quote_asset"
        ));
    }
}
