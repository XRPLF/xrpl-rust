// xrpl.js reference: xrpl.js/packages/xrpl/test/integration/transactions/xchainAddClaimAttestation.test.ts
//
// Scenarios:
//   - base: witness submits a claim attestation for a transfer of 10 XRP.
//           The attestation payload is binary-encoded and signed with the witness private key,
//           matching the same flow as xrpl.js: encode(attestationToSign) → sign(encoded, privateKey).
//
// NOTE: XChainAddClaimAttestation has NO flags; standard 9 common-field order.
//
// Attestation signing flow (mirrors ripple-binary-codec + ripple-keypairs in xrpl.js):
//   1. Build a struct with the attestation fields (PascalCase serde names).
//   2. Binary-encode with xrpl::core::binarycodec::encode  → hex string.
//   3. Hex-decode to bytes.
//   4. Sign bytes with xrpl::core::keypairs::sign using the witness private key.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use crate::common::xchain::setup_bridge;
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::core::binarycodec::encode;
use xrpl::core::keypairs::sign;
use xrpl::models::transactions::xchain_add_claim_attestation::XChainAddClaimAttestation;
use xrpl::models::transactions::xchain_create_claim_id::XChainCreateClaimID;
use xrpl::models::{Amount, Currency, XChainBridge, XRPAmount, XRP};
use xrpl::wallet::Wallet;
use serde::Serialize;

/// Partial attestation payload that gets binary-encoded and signed.
/// Field names are explicitly renamed to match XRPL canonical names.
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
async fn test_xchain_add_claim_attestation_base() {
    with_blockchain_lock(|| async {
        let bridge_setup = setup_bridge().await;
        let client = get_client().await;

        // Claim ID holder (funded wallet on the issuing chain)
        let holder = generate_funded_wallet().await;

        // Source on the "other" (locking) chain — unfunded, just needs a valid address
        let other_seed = xrpl::core::keypairs::generate_seed(None).expect("seed");
        let other_wallet = Wallet::new(&other_seed, 0).expect("wallet");

        // Step 1: XChainCreateClaimID — reserves claim ID 1
        let mut claim_id_tx = XChainCreateClaimID::new(
            holder.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            other_wallet.classic_address.clone().into(),
            XRPAmount::from(&bridge_setup.signature_reward),
            bridge_setup.bridge(),
        );
        submit_and_wait(
            &mut claim_id_tx,
            client,
            Some(&holder),
            Some(true),
            Some(true),
        )
        .await
        .expect("XChainCreateClaimID failed");

        // Step 2: Build + sign the attestation payload
        let attestation = ClaimAttestation {
            xchain_bridge: XChainBridge {
                issuing_chain_door: crate::common::constants::GENESIS_ACCOUNT.into(),
                issuing_chain_issue: Currency::XRP(XRP::new()),
                locking_chain_door: bridge_setup.door_wallet.classic_address.as_str().into(),
                locking_chain_issue: Currency::XRP(XRP::new()),
            },
            other_chain_source: other_wallet.classic_address.as_str(),
            amount: "10000000", // 10 XRP in drops
            attestation_reward_account: bridge_setup.witness_wallet.classic_address.as_str(),
            was_locking_chain_send: 0,
            xchain_claim_id: 1,
        };

        let encoded_hex = encode(&attestation).expect("encode attestation failed");
        let encoded_bytes = hex::decode(&encoded_hex).expect("hex decode failed");
        let attestation_sig = sign(&encoded_bytes, &bridge_setup.witness_wallet.private_key)
            .expect("sign attestation failed");

        // Step 3: XChainAddClaimAttestation — witness submits the signed attestation
        let mut tx = XChainAddClaimAttestation::new(
            bridge_setup.witness_wallet.classic_address.clone().into(),
            None, None, None, None, None, None, None, None,
            Amount::XRPAmount(XRPAmount::from("10000000")), // amount
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_reward_account
            bridge_setup.witness_wallet.classic_address.clone().into(), // attestation_signer_account
            other_wallet.classic_address.clone().into(),                // other_chain_source
            bridge_setup.witness_wallet.public_key.clone().into(),      // public_key
            attestation_sig.into(),                                     // signature
            0,                                                           // was_locking_chain_send
            bridge_setup.bridge(),
            "1".into(), // xchain_claim_id
            None,       // destination (not included in base test)
        );

        let result = submit_and_wait(
            &mut tx,
            client,
            Some(&bridge_setup.witness_wallet),
            Some(true),
            Some(true),
        )
        .await
        .expect("Failed to submit XChainAddClaimAttestation");

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
