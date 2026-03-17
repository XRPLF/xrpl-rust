// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainClaim.test.ts
//
// Scenarios:
//   - base: full cross-chain claim flow:
//       1. XChainCreateClaimID  — destination reserves claim ID 1, paying signature_reward
//       2. XChainAddClaimAttestation — witness attests to a 10 XRP transfer (NO Destination)
//       3. XChainClaim          — destination explicitly claims the 10 XRP
//
// The attestation does NOT include `Destination` — matching the xrpl.js test exactly.
// When no Destination is in the attestation, rippled does NOT auto-deliver on quorum;
// the claimant must submit XChainClaim to specify the destination.
//
// NOTE: XChainClaim has NO flags; standard 9 common-field order.
// xchain_claim_id is Cow<str>.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, test_transaction, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::core::binarycodec::encode;
use xrpl::core::keypairs::sign;
use xrpl::models::transactions::xchain_add_claim_attestation::XChainAddClaimAttestation;
use xrpl::models::transactions::xchain_claim::XChainClaim;
use xrpl::models::transactions::xchain_create_claim_id::XChainCreateClaimID;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};
use xrpl::wallet::Wallet;
use serde::Serialize;

/// Attestation payload for XChainAddClaimAttestation (no Destination — mirrors xrpl.js).
#[derive(Serialize)]
struct ClaimAttestation<'a> {
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
        let other_seed = xrpl::core::keypairs::generate_seed(None, None).expect("seed");
        let other_wallet = Wallet::new(&other_seed, 0).expect("wallet");

        // Step 1: XChainCreateClaimID — destination reserves claim ID 1
        let mut claim_id_tx = XChainCreateClaimID::new(
            destination.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            other_wallet.classic_address.clone().into(),
            XRPAmount::from(bridge_setup.signature_reward.as_str()),
            bridge_setup.bridge(),
        );
        sign_and_submit(
            &mut claim_id_tx,
            client,
            &destination,
            true,
            true,
        )
        .await
        .expect("XChainCreateClaimID failed");

        ledger_accept().await;

        // Step 2: Build + sign attestation payload (NO Destination — matches xrpl.js)
        let attestation = ClaimAttestation {
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
        };

        let encoded_hex = encode(&attestation).expect("encode attestation failed");
        let encoded_bytes = hex::decode(&encoded_hex).expect("hex decode failed");
        let attestation_sig = sign(&encoded_bytes, &bridge_setup.witness_wallet.private_key)
            .expect("sign attestation failed");

        // Step 3: XChainAddClaimAttestation — witness submits (no Destination)
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
            "1".into(),  // xchain_claim_id
            None,        // no destination — claim ID stays alive for XChainClaim
        );
        sign_and_submit(
            &mut attest_tx,
            client,
            &bridge_setup.witness_wallet,
            true,
            true,
        )
        .await
        .expect("XChainAddClaimAttestation failed");

        ledger_accept().await;

        // Step 4: XChainClaim — destination explicitly claims the 10 XRP
        let mut claim_tx = XChainClaim::new(
            destination.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Amount::XRPAmount(XRPAmount::from(amount_drops)),  // amount
            destination.classic_address.clone().into(),        // destination
            bridge_setup.bridge(),
            "1".into(), // xchain_claim_id
            None,       // destination_tag
        );

        test_transaction(&mut claim_tx, &destination).await;
    })
    .await;
}
