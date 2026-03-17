// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/signerListSet.test.ts
//
// Scenarios:
//   - add:    set a signer list with two signers and quorum 2
//   - remove: clear the signer list by setting SignerQuorum to 0

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::signer_list_set::{SignerEntry, SignerListSet};

#[tokio::test]
async fn test_signer_list_set_add() {
    with_blockchain_lock(|| async {
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

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

#[tokio::test]
async fn test_signer_list_set_remove() {
    with_blockchain_lock(|| async {
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

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}
