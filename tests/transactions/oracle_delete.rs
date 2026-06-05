// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleDelete transaction
//
// NOTE: OracleDelete requires a live rippled with amendment support for price
// oracles (XLS-47). These tests validate type construction and serialization
// without submitting to a network.

use crate::common::{
    generate_funded_wallet, get_ledger_close_time, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_objects::AccountObjects as AccountObjectsResult;
use xrpl::models::transactions::oracle_delete::OracleDelete;
use xrpl::models::transactions::oracle_set::OracleSet;
use xrpl::models::transactions::{CommonFields, PriceData, TransactionType};

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

#[tokio::test]
async fn test_oracle_delete_submit() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        // OracleSet LastUpdateTime is POSIX/Unix time. The ledger response uses
        // Ripple epoch seconds, so convert before submitting.
        let last_update_time = (get_ledger_close_time().await + 946_684_800) as u32;
        let oracle_document_id = 2;

        let mut oracle_set = OracleSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            oracle_document_id,
            // Provider is a Blob, so it must be hex-encoded ("chainlink").
            Some("636861696E6C696E6B".into()),
            Some("68747470733A2F2F6578616D706C652E636F6D".into()),
            Some("63757272656E6379".into()),
            last_update_time,
            vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                // AssetPrice is a UInt64 hex string in XRPL binary JSON: 0x2E4 == 740.
                asset_price: Some("2E4".into()),
                scale: Some(1),
            }],
        );
        test_transaction(&mut oracle_set, &wallet).await;

        let mut oracle_delete = OracleDelete::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            oracle_document_id,
        );
        test_transaction(&mut oracle_delete, &wallet).await;

        let client = crate::common::get_client().await;
        let response = client
            .request(
                AccountObjects::new(
                    None,
                    wallet.classic_address.clone().into(),
                    None,
                    Some(LedgerIndex::Str("validated".into())),
                    Some(AccountObjectType::Oracle),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects oracle request failed");
        let result: AccountObjectsResult = response
            .try_into()
            .expect("failed to parse account_objects result");
        assert!(
            result.account_objects.is_empty(),
            "Oracle object should be deleted"
        );
    })
    .await;
}
