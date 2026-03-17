// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/ammBid.test.ts
//
// Scenarios:
//   - base: LP holder bids for the AMM's auction slot (no BidMin/BidMax/AuthAccounts)
//
// NOTE: AMMBid has no flags; uses standard 9 common-field parameter order.

use crate::common::amm::setup_amm_pool;
use crate::common::{test_transaction, with_blockchain_lock};
use xrpl::models::transactions::amm_bid::AMMBid;
use xrpl::models::{Currency, IssuedCurrency, XRP};

#[tokio::test]
async fn test_amm_bid_base() {
    with_blockchain_lock(|| async {
        let pool = setup_amm_pool().await;

        let mut tx = AMMBid::new(
            pool.lp_wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
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
            None, // bid_min
            None, // bid_max
            None, // auth_accounts
        );

        test_transaction(&mut tx, &pool.lp_wallet).await;
    })
    .await;
}
