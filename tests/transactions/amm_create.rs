// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/ammCreate.test.ts
//
// Scenarios:
//   - base: create an XRP/USD AMM pool (250 drops / 250 USD, trading_fee = 12)
//
// Setup (mirrors createAMMPool in xrpl.js/utils.ts):
//   1. issuerWallet AccountSet — enable DefaultRipple
//   2. lpWallet TrustSet      — trust issuer for 1000 USD (tfClearNoRipple)
//   3. issuerWallet Payment   — send 500 USD to lpWallet
//   4. lpWallet AMMCreate     — create the pool (this is the transaction under test)

use crate::common::amm::setup_amm_pool;
use crate::common::{ledger_accept, with_blockchain_lock};

#[tokio::test]
async fn test_amm_create_base() {
    with_blockchain_lock(|| async {
        // setup_amm_pool() runs the full 4-step setup and panics on any failure.
        // If it returns without panicking, AMMCreate succeeded with tesSUCCESS.
        let _pool = setup_amm_pool().await;

        ledger_accept().await;
    })
    .await;
}
