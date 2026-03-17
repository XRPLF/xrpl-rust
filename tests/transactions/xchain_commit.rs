// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainCommit.test.ts
//
// Scenarios:
//   - base: committer locks 10_000_000 drops onto the locking chain door (XChainClaimID = 1)
//
// NOTE: XChainCommit has NO flags; standard 9 common-field order.
// xchain_claim_id is Cow<str> even though it is semantically a number.

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::models::transactions::xchain_commit::XChainCommit;
use xrpl::models::{Amount, XRPAmount};

#[tokio::test]
async fn test_xchain_commit_base() {
    with_blockchain_lock(|| async {
        let bridge = setup_bridge().await;

        // Committer — a separate funded wallet on the locking chain.
        let committer = generate_funded_wallet().await;

        let mut tx = XChainCommit::new(
            committer.classic_address.clone().into(),
            None,                                             // account_txn_id
            None,                                             // fee
            None,                                             // last_ledger_sequence
            None,                                             // memos
            None,                                             // sequence
            None,                                             // signers
            None,                                             // source_tag
            None,                                             // ticket_sequence
            Amount::XRPAmount(XRPAmount::from("10000000")),   // amount: 10 XRP drops
            bridge.bridge(),
            "1".into(),  // xchain_claim_id (Cow<str> representation of the claim ID number)
            None,        // other_chain_destination
        );

        test_transaction(&mut tx, &committer).await;
    })
    .await;
}
