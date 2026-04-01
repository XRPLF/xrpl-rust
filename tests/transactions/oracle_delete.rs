// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleDelete transaction
//
// NOTE: OracleDelete requires a live rippled with amendment support for price
// oracles (XLS-47). These tests validate type construction and serialization
// without submitting to a network.

use xrpl::models::transactions::oracle_delete::OracleDelete;
use xrpl::models::transactions::{CommonFields, TransactionType};

#[test]
fn test_oracle_delete_construction() {
    let oracle_delete = OracleDelete {
        common_fields: CommonFields {
            account: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            transaction_type: TransactionType::OracleDelete,
            fee: Some("12".into()),
            sequence: Some(391),
            ..Default::default()
        },
        oracle_document_id: 1,
    };

    assert_eq!(
        oracle_delete.common_fields.transaction_type,
        TransactionType::OracleDelete
    );
    assert_eq!(oracle_delete.oracle_document_id, 1);
}

#[test]
fn test_oracle_delete_serde_roundtrip() {
    let oracle_delete = OracleDelete::new(
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
    );

    let json = serde_json::to_string(&oracle_delete).unwrap();
    let deserialized: OracleDelete = serde_json::from_str(&json).unwrap();
    assert_eq!(oracle_delete, deserialized);
}
