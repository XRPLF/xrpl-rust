// Scenarios:
//   - base: assign a regular key to a wallet
//   - remove: remove the regular key from a wallet (regular_key = None)

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::transactions::set_regular_key::SetRegularKey;

#[tokio::test]
async fn test_set_regular_key_base() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        // Generate an unfunded wallet to use as the regular key address.
        let seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let key_wallet = xrpl::wallet::Wallet::new(&seed, 0).expect("key wallet");

        let mut tx = SetRegularKey::new(
            wallet.classic_address.clone().into(),
            None,                                            // account_txn_id
            None,                                            // fee
            None,                                            // last_ledger_sequence
            None,                                            // memos
            None,                                            // sequence
            None,                                            // signers
            None,                                            // source_tag
            None,                                            // ticket_sequence
            Some(key_wallet.classic_address.clone().into()), // regular_key
        );

        test_transaction(&mut tx, &wallet).await;
    })
    .await;
}

#[tokio::test]
async fn test_set_regular_key_remove() {
    with_blockchain_lock(|| async {
        let wallet = generate_funded_wallet().await;
        let seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let key_wallet = xrpl::wallet::Wallet::new(&seed, 0).expect("key wallet");

        // Step 1: set a regular key first.
        let mut set_tx = SetRegularKey::new(
            wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(key_wallet.classic_address.clone().into()),
        );
        test_transaction(&mut set_tx, &wallet).await;

        // Step 2: remove the regular key by omitting regular_key.
        let mut remove_tx = SetRegularKey::new(
            wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            None, // regular_key — None removes the key
        );
        test_transaction(&mut remove_tx, &wallet).await;
    })
    .await;
}
