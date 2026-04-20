// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleSet transaction
//
// NOTE: OracleSet requires a live rippled with amendment support for price
// oracles (XLS-47). These tests validate type construction and serialization
// without submitting to a network.

use xrpl::models::transactions::oracle_set::OracleSet;
use xrpl::models::transactions::{CommonFields, PriceData, TransactionType};

#[test]
fn test_oracle_set_construction() {
    let oracle_set = OracleSet {
        common_fields: CommonFields {
            account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            transaction_type: TransactionType::OracleSet,
            fee: Some("12".into()),
            sequence: Some(391),
            ..Default::default()
        },
        oracle_document_id: 1,
        provider: Some("chainlink".into()),
        uri: Some("https://example.com/oracle".into()),
        asset_class: Some("63757272656E6379".into()),
        last_update_time: 743609014,
        price_data_series: Some(vec![PriceData {
            base_asset: Some("EUR".into()),
            quote_asset: Some("USD".into()),
            asset_price: Some("740".into()),
            scale: Some(1),
        }]),
    };

    assert_eq!(
        oracle_set.common_fields.transaction_type,
        TransactionType::OracleSet
    );
    assert_eq!(oracle_set.oracle_document_id, 1);
    assert_eq!(oracle_set.price_data_series.as_ref().unwrap().len(), 1);
}

#[test]
fn test_oracle_set_serde_roundtrip() {
    let oracle_set = OracleSet::new(
        "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
        None,
        Some("12".into()),
        None,
        None,
        Some(391),
        None,
        None,
        None,
        1,
        Some("provider".into()),
        None,
        None,
        743609014,
        None,
    );

    let json = serde_json::to_string(&oracle_set).unwrap();
    let deserialized: OracleSet = serde_json::from_str(&json).unwrap();
    assert_eq!(oracle_set, deserialized);
}
