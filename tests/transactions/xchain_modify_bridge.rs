// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainModifyBridge.test.ts
//
// Scenarios:
//   - base: create a bridge then modify the signature_reward from 200 to 300 drops
//
// NOTE: XChainModifyBridge has `flags` at position 4 (custom flags enum).

use crate::common::{test_transaction, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::models::transactions::xchain_modify_bridge::XChainModifyBridge;
use xrpl::models::{Amount, XRPAmount};

#[tokio::test]
async fn test_xchain_modify_bridge_base() {
    with_blockchain_lock(|| async {
        let bridge = setup_bridge().await;

        // Modify the signature_reward from 200 → 300 drops.
        // XChainModifyBridge has flags at position 4.
        let mut tx = XChainModifyBridge::new(
            bridge.door_wallet.classic_address.clone().into(),
            None,         // account_txn_id
            None,         // fee
            None,         // flags (position 4)
            None,         // last_ledger_sequence
            None,         // memos
            None,         // sequence
            None,         // signers
            None,         // source_tag
            None,         // ticket_sequence
            bridge.bridge(),
            None,                                      // min_account_create_amount
            Some(Amount::XRPAmount(XRPAmount::from("300"))), // signature_reward
        );

        test_transaction(&mut tx, &bridge.door_wallet).await;
    })
    .await;
}
