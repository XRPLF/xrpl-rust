// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainClaim.test.ts
//
// Scenarios:
//   - base: full cross-chain claim flow:
//       1. XChainCreateClaimID  — destination reserves claim ID 1, paying signature_reward
//       2. XChainAddClaimAttestation — witness attests to a 10 XRP transfer (with Destination)
//       3. XChainClaim          — destination claims the 10 XRP
//
// The attestation includes `Destination` because the witness is attesting that
// the transfer should be delivered to `destination.classic_address`.
//
// NOTE: XChainClaim has NO flags; standard 9 common-field order.
// xchain_claim_id is Cow<str>.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::core::binarycodec::encode;
use xrpl::core::keypairs::sign;
use xrpl::models::transactions::xchain_add_claim_attestation::XChainAddClaimAttestation;
use xrpl::models::transactions::xchain_claim::XChainClaim;
use xrpl::models::transactions::xchain_create_claim_id::XChainCreateClaimID;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};
use xrpl::wallet::Wallet;
use serde::Serialize;

/// Attestation payload (with optional Destination) for XChainAddClaimAttestation.
#[derive(Serialize)]
struct ClaimAttestationWithDest<'a> {
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
    #[serde(rename = "XChainClaimID")]
    xchain_claim_id: u64,
    #[serde(rename = "Destination")]
    destination: &'a str,
}

#[tokio::test]
async fn test_xchain_claim_base() {
    with_blockchain_lock(|| async {
        let bridge_setup = setup_bridge().await;
        let client = get_client().await;
        let amount_drops = "10000000"; // 10 XRP

        // Destination — funded wallet on the issuing chain that will receive the funds
        let destination = generate_funded_wallet().await;

        // OtherChainSource — unfunded wallet representing the sender on the locking chain
        let other_seed = xrpl::core::keypairs::generate_seed(None).expect("seed");
        let other_wallet = Wallet::new(&other_seed, 0).expect("wallet");

        // Step 1: XChainCreateClaimID — destination reserves claim ID 1
        let mut claim_id_tx = XChainCreateClaimID::new(
            destination.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            other_wallet.classic_address.clone().into(),
            XRPAmount::from(&bridge_setup.signature_reward),
            bridge_setup.bridge(),
        );
        submit_and_wait(
            &mut claim_id_tx,
            client,
            Some(&destination),
            Some(true),
            Some(true),
        )
        .await
        .expect("XChainCreateClaimID failed");

        // Step 2: Build + sign attestation payload (includes Destination)
        let attestation = ClaimAttestationWithDest {
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
            xchain_claim_id: 1,
            destination: destination.classic_address.as_str(),
        };

        let encoded_hex = encode(&attestation).expect("encode attestation failed");
        let encoded_bytes = hex::decode(&encoded_hex).expect("hex decode failed");
        let attestation_sig = sign(&encoded_bytes, &bridge_setup.witness_wallet.private_key)
            .expect("sign attestation failed");

        // Step 3: XChainAddClaimAttestation — witness submits the signed attestation
        let mut attest_tx = XChainAddClaimAttestation::new(
            bridge_setup.witness_wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Amount::XRPAmount(XRPAmount::from(amount_drops)),
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_reward_account
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_signer_account
            other_wallet.classic_address.clone().into(),                // other_chain_source
            bridge_setup.witness_wallet.public_key.clone().into(),      // public_key
            attestation_sig.into(),                                     // signature
            0,                                                           // was_locking_chain_send
            bridge_setup.bridge(),
            "1".into(),                                                  // xchain_claim_id
            Some(destination.classic_address.clone().into()),           // destination
        );
        submit_and_wait(
            &mut attest_tx,
            client,
            Some(&bridge_setup.witness_wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("XChainAddClaimAttestation failed");

        // Step 4: XChainClaim — destination claims the 10 XRP
        let mut claim_tx = XChainClaim::new(
            destination.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Amount::XRPAmount(XRPAmount::from(amount_drops)),  // amount
            destination.classic_address.clone().into(),        // destination
            bridge_setup.bridge(),
            "1".into(), // xchain_claim_id
            None,       // destination_tag
        );

        let result = submit_and_wait(
            &mut claim_tx,
            client,
            Some(&destination),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit XChainClaim");

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
