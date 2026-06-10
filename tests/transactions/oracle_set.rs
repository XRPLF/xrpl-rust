// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleSet transaction
//
// NOTE: OracleSet requires a live rippled with amendment support for price
// oracles (XLS-47). These tests validate type construction and serialization
// without submitting to a network.

use crate::common::{
    constants::{ORACLE_ASSET_CLASS, ORACLE_PROVIDER, ORACLE_URI, TEST_ACCOUNT},
    generate_funded_wallet, get_ledger_close_time, submit_tx, test_transaction,
    with_blockchain_lock, SubmitOptions,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::LedgerIndex;
use xrpl::models::results::account_objects::AccountObjects as AccountObjectsResult;
use xrpl::models::transactions::oracle_set::OracleSet;
use xrpl::models::transactions::{CommonFields, PriceData, TransactionType};
use xrpl::models::Model;

#[test]
fn test_oracle_set_construction() {
    let oracle_set = OracleSet {
        common_fields: CommonFields {
            account: TEST_ACCOUNT.into(),
            transaction_type: TransactionType::OracleSet,
            fee: Some("12".into()),
            sequence: Some(391),
            ..Default::default()
        },
        oracle_document_id: 1,
        provider: Some(ORACLE_PROVIDER.into()),
        uri: Some(ORACLE_URI.into()),
        asset_class: Some(ORACLE_ASSET_CLASS.into()),
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
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1234,
            // Provider is a Blob, so it must be hex-encoded ("chainlink").
            provider: Some(ORACLE_PROVIDER.into()),
            uri: Some(ORACLE_URI.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
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
        assert_eq!(oracle["Provider"], ORACLE_PROVIDER);
        assert_eq!(oracle["AssetClass"], ORACLE_ASSET_CLASS);
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

#[tokio::test]
async fn test_oracle_set_update_and_delete_pair() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1234,
            provider: Some(ORACLE_PROVIDER.into()),
            uri: Some(ORACLE_URI.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
            last_update_time,
            price_data_series: vec![
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "USD".into(),
                    asset_price: Some("2E4".into()),
                    scale: Some(3),
                },
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "EUR".into(),
                    asset_price: Some("2BC".into()), // 700
                    scale: Some(3),
                },
            ],
        };

        test_transaction(&mut oracle_set, &wallet).await;

        let mut oracle_update = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1234,
            provider: Some(ORACLE_PROVIDER.into()),
            uri: Some(ORACLE_URI.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
            last_update_time: last_update_time + 10,
            price_data_series: vec![
                // Update XRP/USD
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "USD".into(),
                    asset_price: Some("2E5".into()), // 741
                    scale: Some(3),
                },
                // Delete XRP/EUR by omitting asset_price and scale
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "EUR".into(),
                    asset_price: None,
                    scale: None,
                },
                // Add XRP/JPY
                PriceData {
                    base_asset: "XRP".into(),
                    quote_asset: "JPY".into(),
                    asset_price: Some("3A98".into()), // 15000
                    scale: Some(3),
                },
            ],
        };

        test_transaction(&mut oracle_update, &wallet).await;

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
        
        let series = oracle["PriceDataSeries"]
            .as_array()
            .expect("PriceDataSeries should be a JSON array in the Oracle ledger entry");
        assert_eq!(series.len(), 2, "Expected 2 pairs after deleting one and adding one");

        // Find XRP/USD pair
        let usd_pair = series
            .iter()
            .find(|p| p["PriceData"]["QuoteAsset"] == "USD")
            .expect("XRP/USD pair should be present after update");
        assert_eq!(
            usd_pair["PriceData"]["AssetPrice"],
            "2e5",
            "XRP/USD AssetPrice should have been updated to 0x2E5"
        );

        // Find XRP/JPY pair
        let jpy_pair = series
            .iter()
            .find(|p| p["PriceData"]["QuoteAsset"] == "JPY")
            .expect("XRP/JPY pair should be present after being added");
        assert_eq!(
            jpy_pair["PriceData"]["AssetPrice"],
            "3a98",
            "XRP/JPY AssetPrice should be 0x3A98 (15000)"
        );

        // Ensure XRP/EUR is deleted
        let eur_pair = series
            .iter()
            .find(|p| p["PriceData"]["QuoteAsset"] == "EUR");
        assert!(eur_pair.is_none(), "XRP/EUR should have been deleted by omitting AssetPrice/Scale");
    })
    .await;
}

#[tokio::test]
async fn test_oracle_set_tec_token_pair_not_found() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

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
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: None,
        };
        test_transaction(&mut oracle_set, &wallet).await;

        // Try to delete a token pair (XRP/EUR) that doesn't exist
        let mut oracle_update = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1,
            last_update_time: last_update_time + 10,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "EUR".into(),
                asset_price: None,
                scale: None,
            }],
            provider: None,
            asset_class: None,
            uri: None,
        };

        let engine_result = submit_tx(
            &mut oracle_update,
            SubmitOptions {
                wallet: &wallet,
                autofill: true,
                check_fee: true,
            },
        )
        .await;

        assert_eq!(
            engine_result,
            "tecTOKEN_PAIR_NOT_FOUND",
            "Deleting a pair that does not exist should return tecTOKEN_PAIR_NOT_FOUND"
        );
        // Advance the ledger so the consumed sequence number is finalised and
        // does not contaminate subsequent tests.
        crate::common::ledger_accept().await;
    })
    .await;
}

#[tokio::test]
async fn test_oracle_set_tec_array_empty() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

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
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: None,
        };
        test_transaction(&mut oracle_set, &wallet).await;

        // Try to delete the only existing token pair (leaves array empty)
        let mut oracle_update = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1,
            last_update_time: last_update_time + 10,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                asset_price: None,
                scale: None,
            }],
            provider: None,
            asset_class: None,
            uri: None,
        };

        let engine_result = submit_tx(
            &mut oracle_update,
            SubmitOptions {
                wallet: &wallet,
                autofill: true,
                check_fee: true,
            },
        )
        .await;

        assert_eq!(
            engine_result,
            "tecARRAY_EMPTY",
            "Deleting the last pair should return tecARRAY_EMPTY (PriceDataSeries cannot be left empty)"
        );
        crate::common::ledger_accept().await;
    })
    .await;
}

#[tokio::test]
async fn test_oracle_set_tec_invalid_update_time() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

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
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: None,
        };
        test_transaction(&mut oracle_set, &wallet).await;

        // Try to update with a LastUpdateTime that is older than the one in the ledger
        let mut oracle_update = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 1,
            last_update_time: last_update_time - 10,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                asset_price: Some("2E5".into()),
                scale: Some(3),
            }],
            provider: None,
            asset_class: None,
            uri: None,
        };

        let engine_result = submit_tx(
            &mut oracle_update,
            SubmitOptions {
                wallet: &wallet,
                autofill: true,
                check_fee: true,
            },
        )
        .await;

        assert_eq!(
            engine_result,
            "tecINVALID_UPDATE_TIME",
            "LastUpdateTime older than the stored value should return tecINVALID_UPDATE_TIME"
        );
        crate::common::ledger_accept().await;
    })
    .await;
}

#[tokio::test]
async fn test_oracle_set_document_id_zero() {
    // OracleDocumentID is a UInt32; rippled accepts 0 as a valid value.
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 0,
            provider: Some(ORACLE_PROVIDER.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
            last_update_time,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: None,
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
            .expect("account_objects oracle (doc_id=0) request failed");

        let result: AccountObjectsResult = response
            .try_into()
            .expect("failed to parse account_objects result");
        assert_eq!(
            result.account_objects.len(),
            1,
            "Oracle with OracleDocumentID=0 should exist in ledger"
        );
        assert_eq!(result.account_objects[0]["LedgerEntryType"], "Oracle");
    })
    .await;
}

#[tokio::test]
async fn test_oracle_set_uri_max_byte_boundary() {
    // URI is a Blob capped at 256 decoded bytes (kMaxOracleUri = kMaxOracleProvider = 256).
    // Exactly 256 decoded bytes = 512 hex chars on the wire. rippled must accept it.
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32");

        // 256 bytes of 0xAB = 512 'A'+'B' hex chars — exactly at the limit.
        let max_uri = "AB".repeat(256);

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
                asset_price: Some("2E4".into()),
                scale: Some(3),
            }],
            uri: Some(max_uri.as_str().into()),
        };
        // Model-level validation must accept the boundary value.
        oracle_set
            .get_errors()
            .expect("256-byte URI should pass model validation");
        // And rippled must accept it on-chain.
        test_transaction(&mut oracle_set, &wallet).await;
    })
    .await;
}
