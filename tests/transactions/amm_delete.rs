// AMMDelete is a cleanup operation
// for AMMs that could not be fully deleted by AMMWithdraw due to too many trust
// lines (> ~512 LP token holders). In the simple 2-trust-line setup used here,
// AMMWithdraw TfWithdrawAll auto-deletes the AMM in a single transaction, so
// AMMDelete is never needed.
//
// Scenarios:
//   - not_empty: submit AMMDelete against a live (non-empty) AMM to confirm the
//     transaction is correctly rejected with tecAMM_NOT_EMPTY. This validates
//     the transaction type can be built, signed, and submitted, and that rippled
//     applies the right validation rule.
//
// NOTE: tesSUCCESS for AMMDelete requires an AMM that has been fully drained but
// still has ledger objects (trust lines) left over from a TfWithdrawAll that hit
// the per-transaction trust line deletion budget. Reproducing that condition needs
// 512+ LP token holders, which is impractical in a simple integration test.

use crate::common::amm::setup_amm_pool;
use crate::common::{get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::transactions::amm_delete::AMMDelete;
use xrpl::models::{Currency, IssuedCurrency, XRP};

#[tokio::test]
async fn test_amm_delete_not_empty() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let pool = setup_amm_pool().await;

        // Submit AMMDelete against the live (non-empty) pool.
        // rippled should reject it with tecAMM_NOT_EMPTY because the pool
        // still holds assets.  This confirms the transaction can be submitted
        // and that the correct validation is applied.
        let mut tx = AMMDelete::new(
            pool.lp_wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            Currency::XRP(XRP::new()),
            Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                pool.issuer_wallet.classic_address.clone().into(),
            )),
        );

        let result = sign_and_submit(&mut tx, client, &pool.lp_wallet, true, true)
            .await
            .expect("AMMDelete sign_and_submit failed");

        assert_eq!(
            result.engine_result, "tecAMM_NOT_EMPTY",
            "Expected tecAMM_NOT_EMPTY (AMM still has assets) but got: {} — {}",
            result.engine_result, result.engine_result_message
        );

        ledger_accept().await;
    })
    .await;
}
