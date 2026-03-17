// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/ammDeposit.test.ts
//
// Scenarios:
//   - single_asset: deposit 1000 XRP drops into an XRP/USD pool (TfSingleAsset flag)
//
// NOTE: AMMDeposit has `flags` at parameter position 4 (same as NFTokenMint,
// PaymentChannelClaim, AccountSet, TrustSet, Payment).

use crate::common::amm::setup_amm_pool;
use crate::common::{test_transaction, with_blockchain_lock};
use xrpl::models::transactions::amm_deposit::{AMMDeposit, AMMDepositFlag};
use xrpl::models::{Amount, Currency, IssuedCurrency, XRPAmount, XRP};

#[tokio::test]
async fn test_amm_deposit_single_asset() {
    with_blockchain_lock(|| async {
        let pool = setup_amm_pool().await;

        // Deposit 1000 XRP drops into the XRP side of the pool (TfSingleAsset).
        // flags is at parameter position 4.
        let mut tx = AMMDeposit::new(
            pool.lp_wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            Some(vec![AMMDepositFlag::TfSingleAsset].into()), // flags (position 4)
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
            Some(Amount::XRPAmount(XRPAmount::from("1000"))), // amount: 1000 drops
            None, // amount2
            None, // e_price
            None, // lp_token_out
        );

        test_transaction(&mut tx, &pool.lp_wallet).await;
    })
    .await;
}
