// xrpl.js reference: n/a (XLS-47 price oracle support)
//
// Scenarios:
//   - base: construct and validate an OracleDelete transaction
//
// NOTE: OracleDelete requires a live rippled with amendment support for price
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
use xrpl::models::transactions::oracle_delete::OracleDelete;
use xrpl::models::transactions::oracle_set::OracleSet;
use xrpl::models::transactions::{CommonFields, PriceData, TransactionType};

#[test]
fn test_oracle_delete_construction() {
    let oracle_delete = OracleDelete {
        common_fields: CommonFields {
            account: TEST_ACCOUNT.into(),
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

#[tokio::test]
async fn test_oracle_delete_submit() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        // OracleSet LastUpdateTime is POSIX/Unix time. The ledger response uses
        // Ripple epoch seconds, so convert before submitting.
        let last_update_time = u32::try_from(get_ledger_close_time().await + 946_684_800)
            .expect("LastUpdateTime overflows u32: ledger close time is too far in the future");
        let oracle_document_id = 2;

        let mut oracle_set = OracleSet {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleSet,
                ..Default::default()
            },
            oracle_document_id: 2,
            // Provider is a Blob, so it must be hex-encoded ("chainlink").
            provider: Some(ORACLE_PROVIDER.into()),
            uri: Some(ORACLE_URI.into()),
            asset_class: Some(ORACLE_ASSET_CLASS.into()),
            last_update_time,
            price_data_series: vec![PriceData {
                base_asset: "XRP".into(),
                quote_asset: "USD".into(),
                // AssetPrice is a UInt64 hex string in XRPL binary JSON: 0x2E4 == 740.
                asset_price: Some("2E4".into()),
                scale: Some(1),
            }],
        };
        test_transaction(&mut oracle_set, &wallet).await;

        let mut oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id,
        };
        test_transaction(&mut oracle_delete, &wallet).await;

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
        assert!(
            result.account_objects.is_empty(),
            "Oracle object should be deleted"
        );
    })
    .await;
}

#[tokio::test]
async fn test_oracle_delete_not_found() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;

        let mut oracle_delete = OracleDelete {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::OracleDelete,
                ..Default::default()
            },
            oracle_document_id: 999, // Does not exist
        };

        let engine_result = submit_tx(
            &mut oracle_delete,
            SubmitOptions {
                wallet: &wallet,
                autofill: true,
                check_fee: true,
            },
        )
        .await;

        assert_eq!(
            engine_result, "tecNO_ENTRY",
            "Deleting a non-existent Oracle (doc_id=999) should return tecNO_ENTRY"
        );
        crate::common::ledger_accept().await;
    })
    .await;
}
