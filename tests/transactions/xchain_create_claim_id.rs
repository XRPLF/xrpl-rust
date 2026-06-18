// Scenarios:
//   - base: claim ID holder creates a claim ID on an existing bridge
//
// NOTE: XChainCreateClaimID has NO flags; standard 9 common-field order.
// Requires a bridge to already exist (uses setup_bridge helper).

use crate::common::xchain::setup_bridge;
use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::xchain_create_claim_id::XChainCreateClaimID;
use xrpl::models::XRPAmount;
use xrpl::wallet::Wallet;

#[tokio::test]
async fn test_xchain_create_claim_id_base() {
    with_blockchain_lock(|| async {
        let bridge = setup_bridge().await;

        // Holder (claim ID creator) — a separate funded wallet on the issuing chain.
        let holder = generate_funded_wallet().await;

        // OtherChainSource is a wallet address on the "other" (locking) chain.
        // In standalone mode it is unfunded; the address just needs to be valid.
        let other_seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let other_wallet = Wallet::new(&other_seed, 0).expect("wallet");

        let mut tx = XChainCreateClaimID::new(
            holder.classic_address.clone().into(),
            None,                                              // account_txn_id
            None,                                              // fee
            None,                                              // last_ledger_sequence
            None,                                              // memos
            None,                                              // sequence
            None,                                              // signers
            None,                                              // source_tag
            None,                                              // ticket_sequence
            other_wallet.classic_address.clone().into(),       // other_chain_source
            XRPAmount::from(bridge.signature_reward.as_str()), // signature_reward
            bridge.bridge(),
        );

        test_transaction(&mut tx, &holder).await;
    })
    .await;
}
