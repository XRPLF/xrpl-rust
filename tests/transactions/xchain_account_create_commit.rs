// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainAccountCreateCommit.test.ts
//
// Scenarios:
//   - base: committer funds creation of a new account on the issuing chain by locking
//           10_000_000 drops + signature_reward on the locking chain door
//
// NOTE: XChainAccountCreateCommit has NO flags; standard 9 common-field order.
// The `destination` address does not need to be funded (it will be created on the issuing chain).

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::xchain_account_create_commit::XChainAccountCreateCommit;
use xrpl::models::{Amount, XRPAmount};
use xrpl::wallet::Wallet;

#[tokio::test]
async fn test_xchain_account_create_commit_base() {
    with_blockchain_lock(|| async {
        let bridge = setup_bridge().await;
        let client = get_client().await;

        // Committer — a funded wallet on the locking chain.
        let committer = generate_funded_wallet().await;

        // Destination — an unfunded address that will be created on the issuing chain.
        let dest_seed = xrpl::core::keypairs::generate_seed(None).expect("seed");
        let dest_wallet = Wallet::new(&dest_seed, 0).expect("wallet");

        let mut tx = XChainAccountCreateCommit::new(
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
            dest_wallet.classic_address.clone().into(),       // destination
            bridge.bridge(),
            Some(Amount::XRPAmount(XRPAmount::from(&bridge.signature_reward))), // signature_reward
        );

        let result = submit_and_wait(
            &mut tx,
            client,
            Some(&committer),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit XChainAccountCreateCommit");

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
