// Scenarios:
//   - base: witness submits an account-create attestation for a 300 XRP transfer
//           to a new (unfunded) destination address.
//
// NOTE: XChainAddAccountCreateAttestation has NO flags; standard 9 common-field order.
//
// The attestation payload differs from XChainAddClaimAttestation:
//   it includes XChainAccountCreateCount and SignatureReward instead of XChainClaimID.

use crate::common::xchain::setup_bridge;
use crate::common::{test_transaction, with_blockchain_lock};
use serde::Serialize;
use xrpl::core::binarycodec::encode;
use xrpl::core::keypairs::sign;
use xrpl::models::transactions::xchain_add_account_create_attestation::XChainAddAccountCreateAttestation;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};
use xrpl::wallet::Wallet;

/// Attestation payload for XChainAddAccountCreateAttestation.
#[derive(Serialize)]
struct AccountCreateAttestation<'a> {
    #[serde(rename = "XChainBridge")]
    xchain_bridge: XChainBridge<'a>,
    #[serde(rename = "OtherChainSource")]
    other_chain_source: &'a str,
    #[serde(rename = "Amount")]
    amount: &'a str,
    #[serde(rename = "AttestationRewardAccount")]
    attestation_reward_account: &'a str,
    #[serde(rename = "WasLockingChainSend")]
    was_locking_chain_send: u8,
    #[serde(rename = "XChainAccountCreateCount")]
    xchain_account_create_count: u64,
    #[serde(rename = "Destination")]
    destination: &'a str,
    #[serde(rename = "SignatureReward")]
    signature_reward: &'a str,
}

#[tokio::test]
async fn test_xchain_add_account_create_attestation_base() {
    with_blockchain_lock(|| async {
        let bridge_setup = setup_bridge().await;
        let amount_drops = "300000000"; // 300 XRP in drops (xrpToDrops(300))

        // Source on the "other" (locking) chain — unfunded
        let other_seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let other_wallet = Wallet::new(&other_seed, 0).expect("wallet");

        // Destination — a new account to be created on the issuing chain (unfunded)
        let dest_seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let dest_wallet = Wallet::new(&dest_seed, 0).expect("wallet");

        // Build + sign the attestation payload
        let attestation = AccountCreateAttestation {
            xchain_bridge: XChainBridge {
                issuing_chain_door: crate::common::constants::GENESIS_ACCOUNT.into(),
                issuing_chain_issue: Currency::XRP(XRP::new()),
                locking_chain_door: bridge_setup.door_wallet.classic_address.as_str().into(),
                locking_chain_issue: Currency::XRP(XRP::new()),
            },
            other_chain_source: other_wallet.classic_address.as_str(),
            amount: amount_drops,
            attestation_reward_account: bridge_setup.witness_wallet.classic_address.as_str(),
            was_locking_chain_send: 0,
            xchain_account_create_count: 1,
            destination: dest_wallet.classic_address.as_str(),
            signature_reward: &bridge_setup.signature_reward,
        };

        let encoded_hex = encode(&attestation).expect("encode attestation failed");
        let encoded_bytes = hex::decode(&encoded_hex).expect("hex decode failed");
        let attestation_sig = sign(&encoded_bytes, &bridge_setup.witness_wallet.private_key)
            .expect("sign attestation failed");

        // XChainAddAccountCreateAttestation — witness submits the signed attestation
        let mut tx = XChainAddAccountCreateAttestation::new(
            bridge_setup.witness_wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from(amount_drops)), // amount
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_reward_account
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_signer_account
            dest_wallet.classic_address.clone().into(),                 // destination
            other_wallet.classic_address.clone().into(),                // other_chain_source
            bridge_setup.witness_wallet.public_key.clone().into(),      // public_key
            attestation_sig.into(),                                     // signature
            Amount::XRPAmount(XRPAmount::from(bridge_setup.signature_reward.as_str())), // signature_reward
            0,          // was_locking_chain_send
            "1".into(), // xchain_account_create_count
            bridge_setup.bridge(),
        );

        test_transaction(&mut tx, &bridge_setup.witness_wallet).await;
    })
    .await;
}
