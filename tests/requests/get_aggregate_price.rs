// Scenarios:
//   - base: create one oracle with XRP/USD price and query get_aggregate_price
//   - trim: create three oracles and query with trim parameter

use crate::common::{
    constants::{ORACLE_ASSET_CLASS, ORACLE_PROVIDER},
    generate_funded_wallet, get_ledger_close_time, test_transaction, with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::get_aggregate_price::{GetAggregatePrice, OracleDescriptor};
use xrpl::models::results::get_aggregate_price::GetAggregatePrice as GetAggregatePriceResult;
use xrpl::models::transactions::oracle_set::OracleSet;
use xrpl::models::transactions::{CommonFields, PriceData, TransactionType};

#[tokio::test]
async fn test_get_aggregate_price_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32");

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1,
            provider: Some(ORACLE_PROVIDER.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
            last_update_time,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                // 0x2E4 = 740; with scale=3 represents 0.740
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: None,
        };
        test_transaction(&mut oracle_set, &wallet).await;

        let client = crate::common::get_client().await;
        let request = GetAggregatePrice::new(
            None,
            "XRP".into(),
            "USD".into(),
            vec![OracleDescriptor {
                account: wallet.classic_address.clone().into(),
                oracle_document_id: 1,
            }],
            None,
            None,
        );

        let response = client
            .request(request.into())
            .await
            .expect("get_aggregate_price request failed");

        let result: GetAggregatePriceResult = response
            .try_into()
            .expect("failed to parse get_aggregate_price result");

        assert!(
            !result.entire_set.mean.is_empty(),
            "mean should be non-empty"
        );
        assert_eq!(result.entire_set.size, 1, "one oracle = size 1");
        assert!(!result.median.is_empty(), "median should be non-empty");
        assert!(result.time > 0, "time should be positive");
        assert!(result.ledger_current_index > 0);
        assert!(result.trimmed_set.is_none(), "no trim requested");
    })
    .await;
}

#[tokio::test]
async fn test_get_aggregate_price_with_trim() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;

        // Create three separate oracle accounts with different XRP/USD prices.
        let prices: &[(&str, u8)] = &[
            ("2BC", 3), // 0x2BC = 700 → 0.700
            ("2E4", 3), // 0x2E4 = 740 → 0.740
            ("30C", 3), // 0x30C = 780 → 0.780
        ];

        let mut oracles: Vec<OracleDescriptor> = Vec::new();
        for (i, (price_hex, scale)) in prices.iter().enumerate() {
            let wallet = generate_funded_wallet().await;
            let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
                .expect("LastUpdateTime overflows u32");

            let doc_id = u32::try_from(i + 1).unwrap();
            let mut oracle_set = OracleSet {
                common_fields: CommonFields {
                    account: wallet.classic_address.clone().into(),
                    transaction_type: TransactionType::OracleSet,
                    ..Default::default()
                },
                oracle_document_id: doc_id,
                provider: Some(ORACLE_PROVIDER.into()),
                asset_class: Some(ORACLE_ASSET_CLASS.into()),
                last_update_time,
                price_data_series: vec![PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "USD".into(),
                    asset_price: Some((*price_hex).into()),
                    scale: Some(*scale),
                }],
                uri: None,
            };
            test_transaction(&mut oracle_set, &wallet).await;
            oracles.push(OracleDescriptor {
                account: wallet.classic_address.clone().into(),
                oracle_document_id: doc_id,
            });
        }

        let request = GetAggregatePrice::new(
            None,
            "XRP".into(),
            "USD".into(),
            oracles,
            Some(20), // trim 20% of outliers
            None,
        );

        let response = client
            .request(request.into())
            .await
            .expect("get_aggregate_price (with trim) request failed");

        let result: GetAggregatePriceResult = response
            .try_into()
            .expect("failed to parse get_aggregate_price (trim) result");

        assert_eq!(result.entire_set.size, 3, "three oracles = size 3");
        assert!(!result.entire_set.mean.is_empty());
        assert!(!result.median.is_empty());

        let trimmed = result
            .trimmed_set
            .expect("trimmed_set should be present when trim is specified");
        assert!(trimmed.size <= 3, "trimmed set cannot exceed full set");
        assert!(!trimmed.mean.is_empty());
    })
    .await;
}
