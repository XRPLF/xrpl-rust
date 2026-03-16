// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/signerListSet.test.ts
//
// Scenarios:
//   - add:    set a signer list with two signers and quorum 2
//   - remove: clear the signer list by setting SignerQuorum to 0

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::signer_list_set::{SignerEntry, SignerListSet};

#[tokio::test]
async fn test_signer_list_set_add() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = SignerListSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            2, // signer_quorum
            Some(vec![
                SignerEntry::new("r5nx8ZkwEbFztnc8Qyi22DE9JYjRzNmvs".to_string(), 1),
                SignerEntry::new("r3RtUvGw9nMoJ5FuHxuoVJvcENhKtuF9ud".to_string(), 1),
            ]),
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit SignerListSet (add)");

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

#[tokio::test]
async fn test_signer_list_set_remove() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        // Use a fresh wallet that hasn't had a signer list set so this test is self-contained.
        // Setting SignerQuorum = 0 with no SignerEntries deletes any existing signer list.
        let wallet = generate_funded_wallet().await;

        let mut tx = SignerListSet::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            0,    // signer_quorum = 0 removes the signer list
            None, // no signer_entries
        );

        let result = submit_and_wait(&mut tx, client, Some(&wallet), Some(true), Some(true))
            .await
            .expect("Failed to submit SignerListSet (remove)");

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
