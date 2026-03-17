// Shared XChain bridge setup helper used by all XChain integration tests.
//
// Creates a minimal XRP/XRP cross-chain bridge in standalone/testnet mode:
//   1. door_wallet:    XChainCreateBridge — locking_chain_door = door_wallet,
//                                           issuing_chain_door = GENESIS_ACCOUNT,
//                                           signature_reward = 200 drops,
//                                           min_account_create_amount = 10_000_000 drops
//   2. door_wallet:    SignerListSet       — registers witness_wallet as the sole signer (quorum=1)
//
// The returned `XChainBridgeSetup` carries both wallets.  Individual tests construct
// the `XChainBridge` definition inline via `setup.bridge()`, which borrows from the struct.

use super::{constants::GENESIS_ACCOUNT, generate_funded_wallet, test_transaction};
use xrpl::models::transactions::signer_list_set::{SignerEntry, SignerListSet};
use xrpl::models::transactions::xchain_create_bridge::XChainCreateBridge;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};
use xrpl::wallet::Wallet;

pub struct XChainBridgeSetup {
    /// Locking chain door account; submitted XChainCreateBridge.
    pub door_wallet: Wallet,
    /// The witness signer registered in the bridge's SignerList.
    pub witness_wallet: Wallet,
    /// SignatureReward used when the bridge was created ("200" drops).
    pub signature_reward: String,
}

impl XChainBridgeSetup {
    /// Build an `XChainBridge` value borrowing from this setup.
    ///
    /// locking_chain_door = door_wallet.classic_address
    /// issuing_chain_door = GENESIS_ACCOUNT (has ample XRP on testnet/standalone)
    /// Both issues = XRP
    pub fn bridge(&self) -> XChainBridge<'_> {
        XChainBridge {
            issuing_chain_door: GENESIS_ACCOUNT.into(),
            issuing_chain_issue: Currency::XRP(XRP::new()),
            locking_chain_door: self.door_wallet.classic_address.as_str().into(),
            locking_chain_issue: Currency::XRP(XRP::new()),
        }
    }
}

/// Set up a minimal XRP/XRP bridge with one witness signer.
/// Runs XChainCreateBridge + SignerListSet against the live client.
#[cfg(feature = "std")]
pub async fn setup_bridge() -> XChainBridgeSetup {
    let door_wallet = generate_funded_wallet().await;
    let witness_wallet = generate_funded_wallet().await;
    let signature_reward = "200".to_string();

    // Step 1: XChainCreateBridge — door_wallet is locking_chain_door
    // No flags; standard 9 common-field order.
    let mut bridge_tx = XChainCreateBridge::new(
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
    test_transaction(&mut bridge_tx, &door_wallet).await;

    // Step 2: SignerListSet — register witness_wallet as the sole signer (quorum = 1)
    let mut signer_tx = SignerListSet::new(
        door_wallet.classic_address.clone().into(),
        None, // account_txn_id
        None, // fee
        None, // last_ledger_sequence
        None, // memos
        None, // sequence
        None, // signers
        None, // source_tag
        None, // ticket_sequence
        1,    // signer_quorum
        Some(vec![SignerEntry::new(
            witness_wallet.classic_address.clone(),
            1, // signer_weight
        )]),
    );
    test_transaction(&mut signer_tx, &door_wallet).await;

    XChainBridgeSetup {
        door_wallet,
        witness_wallet,
        signature_reward,
    }
}
