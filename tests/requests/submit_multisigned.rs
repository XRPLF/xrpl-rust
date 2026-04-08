// Scenarios:
//   - base: set up a SignerList on an account, multisign a transaction with
//     two signers, submit via submit_multisigned, and verify tesSUCCESS

use crate::common::with_blockchain_lock;
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::{autofill, sign};
use xrpl::models::requests::submit_multisigned::SubmitMultisigned as SubmitMultisignedRequest;
use xrpl::models::results::submit_multisigned::SubmitMultisigned as SubmitMultisignedResult;
use xrpl::models::transactions::account_set::AccountSet;
use xrpl::models::transactions::signer_list_set::{SignerEntry, SignerListSet};

#[tokio::test]
async fn test_submit_multisigned_base() {
    with_blockchain_lock(|| async {
        let client = crate::common::get_client().await;
        let main_wallet = crate::common::generate_funded_wallet().await;
        let signer1 = crate::common::generate_funded_wallet().await;
        let signer2 = crate::common::generate_funded_wallet().await;

        // Step 1: Set up a SignerList on the main account (quorum=2, each weight=1)
        let mut signer_list_tx = SignerListSet::new(
            main_wallet.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            2,    // signer_quorum
            Some(vec![
                SignerEntry::new(signer1.classic_address.clone(), 1),
                SignerEntry::new(signer2.classic_address.clone(), 1),
            ]),
        );
        crate::common::test_transaction(&mut signer_list_tx, &main_wallet).await;

        // Step 2: Build the transaction to be multisigned (AccountSet to set a domain)
        let mut tx = AccountSet::new(
            main_wallet.classic_address.clone().into(),
            None,                                  // account_txn_id
            None,                                  // fee
            None,                                  // flags
            None,                                  // last_ledger_sequence
            None,                                  // memos
            None,                                  // sequence
            None,                                  // signers
            None,                                  // source_tag
            None,                                  // ticket_sequence
            None,                                  // clear_flag
            Some("6578616d706c652e636f6d".into()), // domain = "example.com" in hex
            None,                                  // email_hash
            None,                                  // message_key
            None,                                  // set_flag
            None,                                  // transfer_rate
            None,                                  // tick_size
            None,                                  // nftoken_minter
        );

        // Autofill without signing
        autofill(&mut tx, client, None)
            .await
            .expect("autofill failed");

        // Set fee high enough for multisig (n+1 signatures * base_fee)
        // For 2 signers: 3 * 10 = 30 drops minimum
        tx.common_fields.fee = Some("30000".into());

        // Step 3: Multisign with both signers
        let mut tx_signer1 = tx.clone();
        let mut tx_signer2 = tx.clone();

        sign(&mut tx_signer1, &signer1, true).expect("multisign signer1 failed");
        sign(&mut tx_signer2, &signer2, true).expect("multisign signer2 failed");

        // Combine signers
        let signers1 = tx_signer1.common_fields.signers.unwrap_or_default();
        let signers2 = tx_signer2.common_fields.signers.unwrap_or_default();
        let mut combined_signers = signers1;
        combined_signers.extend(signers2);

        tx.common_fields.signing_pub_key = Some("".into());
        tx.common_fields.signers = Some(combined_signers);

        // Step 4: Serialize and submit as multisigned
        let tx_json = serde_json::to_value(&tx).expect("serialize tx failed");
        let request = SubmitMultisignedRequest::new(None, tx_json, None);

        let response = client
            .request(request.into())
            .await
            .expect("submit_multisigned request failed");

        let result: SubmitMultisignedResult = response
            .try_into()
            .expect("failed to parse submit_multisigned result");

        assert_eq!(result.engine_result.as_ref(), "tesSUCCESS");
        assert!(!result.tx_blob.is_empty());
    })
    .await;
}
