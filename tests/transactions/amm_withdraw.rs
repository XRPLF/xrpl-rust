// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/ammWithdraw.test.ts
//
// Scenarios:
//   - single_asset: withdraw 500 XRP drops from an XRP/USD pool (TfSingleAsset flag)
//
// NOTE: AMMWithdraw has `flags` at parameter position 4.

use crate::common::amm::setup_amm_pool;
use crate::common::{get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::amm_withdraw::{AMMWithdraw, AMMWithdrawFlag};
use xrpl::models::{Amount, Currency, IssuedCurrency, XRPAmount, XRP};

#[tokio::test]
async fn test_amm_withdraw_single_asset() {
    with_blockchain_lock(|| async {
        let pool = setup_amm_pool().await;
        let client = get_client().await;

        // Withdraw 500 XRP drops from the XRP side of the pool (TfSingleAsset).
        // flags is at parameter position 4.
        let mut tx = AMMWithdraw::new(
            pool.lp_wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            Some(vec![AMMWithdrawFlag::TfSingleAsset].into()), // flags (position 4)
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            Currency::XRP(XRP::new()), // asset
            Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                pool.issuer_wallet.classic_address.clone().into(),
            )), // asset2
            Some(Amount::XRPAmount(XRPAmount::from("500"))), // amount: 500 drops
            None, // amount2
            None, // e_price
            None, // lp_token_in
        );

        let result = submit_and_wait(
            &mut tx,
            client,
            Some(&pool.lp_wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit AMMWithdraw");

        assert_eq!(
            result
                .get_transaction_metadata()
                .expect("Expected metadata")
                .transaction_result,
            "tesSUCCESS"
        );

        ledger_accept().await;
    })
    .await;
}
