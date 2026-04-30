// Scenarios:
//   - base: door account creates an XRP/XRP bridge with genesis as the issuing chain door
//
// NOTE: XChainCreateBridge has NO flags; uses standard 9 common-field parameter order.
// The `account` must be either the locking_chain_door or the issuing_chain_door.
// locking_chain_door == account, issuing_chain_door == GENESIS_ACCOUNT

use crate::common::constants::GENESIS_ACCOUNT;
use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::xchain_create_bridge::XChainCreateBridge;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};

#[tokio::test]
async fn test_xchain_create_bridge_base() {
    with_blockchain_lock(|| async {
        let door_wallet = generate_funded_wallet().await;

        let mut tx = XChainCreateBridge::new(
            door_wallet.classic_address.clone().into(),
            None,                                      // account_txn_id
            None,                                      // fee
            None,                                      // last_ledger_sequence
            None,                                      // memos
            None,                                      // sequence
            None,                                      // signers
            None,                                      // source_tag
            None,                                      // ticket_sequence
            Amount::XRPAmount(XRPAmount::from("200")), // signature_reward
            XChainBridge {
                issuing_chain_door: GENESIS_ACCOUNT.into(),
                issuing_chain_issue: Currency::XRP(XRP::new()),
                locking_chain_door: door_wallet.classic_address.clone().into(),
                locking_chain_issue: Currency::XRP(XRP::new()),
            },
            Some(XRPAmount::from("10000000")), // min_account_create_amount (10 XRP)
        );

        test_transaction(&mut tx, &door_wallet).await;
    })
    .await;
}
