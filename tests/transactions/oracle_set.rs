// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleSet transaction
//
// NOTE: OracleSet requires a live rippled with amendment support for price
// oracles (XLS-47). These tests validate type construction and serialization
// without submitting to a network.

use crate::common::{
    generate_funded_wallet, get_ledger_close_time, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_objects::AccountObjects as AccountObjectsResult;
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
        provider: Some("636861696E6C696E6B".into()),
        uri: Some("68747470733A2F2F6578616D706C652E636F6D".into()),
        asset_class: Some("63757272656E6379".into()),
        last_update_time: 743609014,
        price_data_series: vec![PriceData {
            base_asset: "EUR".into(),
            quote_asset: "USD".into(),
            asset_price: Some("2E4".into()),
            scale: Some(1),
        }],
    };

    assert_eq!(
        oracle_set.common_fields.transaction_type,
        TransactionType::OracleSet
    );
    assert_eq!(oracle_set.oracle_document_id, 1);
    assert_eq!(oracle_set.price_data_series.len(), 1);
}

#[tokio::test]
async fn test_oracle_set_submit() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        // OracleSet LastUpdateTime is POSIX/Unix time. The ledger response uses
        // Ripple epoch seconds, so convert before submitting.
        let last_update_time = (get_ledger_close_time().await + 946_684_800) as u32;

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1234,
            // Provider is a Blob, so it must be hex-encoded ("chainlink").
            provider: Some("636861696E6C696E6B".into()),
            uri: Some("6469645F6578616D706C65".into()),
            asset_class: Some("63757272656E6379".into()),
            last_update_time,
            price_data_series: vec![
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "USD".into(),
                    // AssetPrice is a UInt64 hex string in XRPL binary JSON: 0x2E4 == 740.
                    asset_price: Some("2E4".into()),
                    scale: Some(3),
                },
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "INR".into(),
                    asset_price: Some("7FFFFFFFFFFFFFFF".into()),
                    scale: Some(3),
                },
            ],
        };

        test_transaction(&mut oracle_set, &wallet).await;

        let client = crate::common::get_client().await;
        let response = client
            .request(
                AccountObjects {
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: Some(xrpl::models::requests::LookupByLedgerRequest {
                        ledger_hash: None,
                        ledger_index: Some(LedgerIndex::Str("validated".into())),
                    }),
                    r#type: Some(AccountObjectType::Oracle),
                    common_fields: xrpl::models::requests::CommonFields {
                        command: xrpl::models::requests::RequestMethod::AccountObjects,
                        id: None,
                    },
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects oracle request failed");
        let result: AccountObjectsResult = response
            .try_into()
            .expect("failed to parse account_objects result");
        assert_eq!(result.account_objects.len(), 1);
        let oracle = &result.account_objects[0];
        assert_eq!(oracle["LedgerEntryType"], "Oracle");
        assert_eq!(oracle["Owner"], wallet.classic_address);
        assert_eq!(oracle["Provider"], "636861696E6C696E6B");
        assert_eq!(oracle["AssetClass"], "63757272656E6379");
        assert_eq!(
            oracle["PriceDataSeries"][0]["PriceData"]["BaseAsset"],
            "XRP"
        );
        assert_eq!(
            oracle["PriceDataSeries"][0]["PriceData"]["QuoteAsset"],
            "USD"
        );
        assert_eq!(
            oracle["PriceDataSeries"][0]["PriceData"]["AssetPrice"],
            "2e4"
        );
        assert_eq!(oracle["PriceDataSeries"][0]["PriceData"]["Scale"], 3);
        assert_eq!(
            oracle["PriceDataSeries"][1]["PriceData"]["AssetPrice"],
            "7fffffffffffffff"
        );
    })
    .await;
}
