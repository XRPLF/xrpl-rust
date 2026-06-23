// Clawback integration tests
//
// Scenarios:
//   - base: claw back issued currency from a holder
//
// Prerequisites clawback test (IOU case):
// Note: It is impractical to test the MPT clawback test-cases because xrpl-rust does not have the necessary MPT-related models.
//   1. Fund an issuer wallet and enable AsfAllowTrustLineClawback via AccountSet
//   2. Fund a holder wallet
//   3. Create a trust line from holder to issuer (TrustSet)
//   4. Issue tokens from issuer to holder (Payment)
//   5. Execute the Clawback transaction

use crate::common::{generate_funded_wallet, test_transaction, with_blockchain_lock};
use xrpl::models::{
    transactions::{
        account_set::{AccountSet, AccountSetFlag},
        clawback::Clawback,
        payment::Payment,
        trust_set::TrustSet,
    },
    Amount, IssuedCurrencyAmount,
};

/// Set up the issuer account with the AllowTrustLineClawback flag,
/// create a trust line from holder to issuer, and issue tokens.
/// Returns the issued currency amount identifier for use in Clawback.
async fn setup_clawback_prerequisites(
    issuer: &xrpl::wallet::Wallet,
    holder: &xrpl::wallet::Wallet,
    currency: &str,
    trust_limit: &str,
    issue_amount: &str,
) {
    let currency = currency.to_string();
    let trust_limit = trust_limit.to_string();
    let issue_amount = issue_amount.to_string();

    // Step 1: Enable AllowTrustLineClawback on the issuer account
    let mut account_set = AccountSet::new(
        issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,                                            // clear_flag
        None,                                            // domain
        None,                                            // email_hash
        None,                                            // message_key
        Some(AccountSetFlag::AsfAllowTrustLineClawback), // set_flag
        None,                                            // transfer_rate
        None,                                            // tick_size
        None,                                            // nftoken_minter
    );
    test_transaction(&mut account_set, issuer).await;

    // Step 2: Create trust line from holder to issuer
    let mut trust_set = TrustSet::new(
        holder.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        IssuedCurrencyAmount::new(
            currency.clone().into(),
            issuer.classic_address.clone().into(),
            trust_limit.into(),
        ),
        None,
        None,
    );
    test_transaction(&mut trust_set, holder).await;

    // Step 3: Issue tokens from issuer to holder
    let mut payment = Payment::new(
        issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            currency.into(),
            issuer.classic_address.clone().into(),
            issue_amount.into(),
        )),
        holder.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
    );
    test_transaction(&mut payment, issuer).await;
}

#[tokio::test]
async fn test_clawback_base() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;

        setup_clawback_prerequisites(&issuer, &holder, "USD", "1000", "500").await;

        // Claw back 100 USD from holder.
        // Note: for IOU clawback the Amount issuer field is the *holder* address.
        let mut tx = Clawback::new(
            issuer.classic_address.clone().into(),
            None, // account_txn_id
            None, // fee
            None, // last_ledger_sequence
            None, // memos
            None, // sequence
            None, // signers
            None, // source_tag
            None, // ticket_sequence
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                holder.classic_address.clone().into(),
                "100".into(),
            )),
            None, // holder (must be None for IOU clawback)
        );

        test_transaction(&mut tx, &issuer).await;
    })
    .await;
}
