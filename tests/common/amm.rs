// Shared AMM pool setup helper used by all AMM integration tests.
//
// Creates a minimal XRP/USD AMM pool:
//   1. issuerWallet:  AccountSet — enable DefaultRipple so the IOU can flow
//   2. lpWallet:      TrustSet  — trust issuer for up to 1000 USD (tfClearNoRipple)
//   3. issuerWallet:  Payment   — send 500 USD to lpWallet
//   4. lpWallet:      AMMCreate — 250 XRP drops + 250 USD, trading_fee = 12
//
// The returned `AmmPool` carries both wallets so individual tests can build
// `Currency` / `IssuedCurrencyAmount` values from `issuer_wallet.classic_address`.

use super::{generate_funded_wallet, get_client};
use xrpl::asynch::transaction::submit_and_wait;
use xrpl::models::transactions::account_set::{AccountSet, AccountSetFlag};
use xrpl::models::transactions::amm_create::AMMCreate;
use xrpl::models::transactions::payment::Payment;
use xrpl::models::transactions::trust_set::{TrustSet, TrustSetFlag};
use xrpl::models::{Amount, IssuedCurrencyAmount, XRPAmount};
use xrpl::wallet::Wallet;

pub struct AmmPool {
    pub lp_wallet: Wallet,
    pub issuer_wallet: Wallet,
}

#[cfg(feature = "std")]
pub async fn setup_amm_pool() -> AmmPool {
    let client = get_client().await;
    let issuer_wallet = generate_funded_wallet().await;
    let lp_wallet = generate_funded_wallet().await;

    // Step 1: enable DefaultRipple on issuer so the USD IOU can flow through AMM
    // AccountSet has flags at position 4; set_flag is at position 15.
    let mut set_tx = AccountSet::new(
        issuer_wallet.classic_address.clone().into(),
        None, // account_txn_id
        None, // fee
        None, // flags (position 4)
        None, // last_ledger_sequence
        None, // memos
        None, // sequence
        None, // signers
        None, // source_tag
        None, // ticket_sequence
        None, // clear_flag
        None, // domain
        None, // email_hash
        None, // message_key
        Some(AccountSetFlag::AsfDefaultRipple), // set_flag
        None, // transfer_rate
        None, // tick_size
        None, // nftoken_minter
    );
    submit_and_wait(&mut set_tx, client, Some(&issuer_wallet), Some(true), Some(true))
        .await
        .expect("AMM setup: AccountSet DefaultRipple failed");

    // Step 2: lp_wallet sets trust line to issuer for 1000 USD
    // TrustSet has flags at position 4.
    let mut trust_tx = TrustSet::new(
        lp_wallet.classic_address.clone().into(),
        None, // account_txn_id
        None, // fee
        Some(vec![TrustSetFlag::TfClearNoRipple].into()), // flags (position 4)
        None, // last_ledger_sequence
        None, // memos
        None, // sequence
        None, // signers
        None, // source_tag
        None, // ticket_sequence
        IssuedCurrencyAmount::new(
            "USD".into(),
            issuer_wallet.classic_address.clone().into(),
            "1000".into(),
        ),
        None, // quality_in
        None, // quality_out
    );
    submit_and_wait(&mut trust_tx, client, Some(&lp_wallet), Some(true), Some(true))
        .await
        .expect("AMM setup: TrustSet failed");

    // Step 3: issuer sends 500 USD to lp_wallet
    // Payment has flags at position 4.
    let mut pay_tx = Payment::new(
        issuer_wallet.classic_address.clone().into(),
        None, // account_txn_id
        None, // fee
        None, // flags (position 4)
        None, // last_ledger_sequence
        None, // memos
        None, // sequence
        None, // signers
        None, // source_tag
        None, // ticket_sequence
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            issuer_wallet.classic_address.clone().into(),
            "500".into(),
        )),
        lp_wallet.classic_address.clone().into(), // destination
        None, // destination_tag
        None, // invoice_id
        None, // paths
        None, // send_max
        None, // deliver_min
    );
    submit_and_wait(&mut pay_tx, client, Some(&issuer_wallet), Some(true), Some(true))
        .await
        .expect("AMM setup: Payment USD failed");

    // Step 4: lp_wallet creates the AMM with 250 XRP drops + 250 USD, fee = 12
    // AMMCreate has no flags; uses standard 9 common fields.
    let mut amm_tx = AMMCreate::new(
        lp_wallet.classic_address.clone().into(),
        None, // account_txn_id
        None, // fee
        None, // last_ledger_sequence
        None, // memos
        None, // sequence
        None, // signers
        None, // source_tag
        None, // ticket_sequence
        Amount::XRPAmount(XRPAmount::from("250")), // amount: 250 XRP drops
        Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            issuer_wallet.classic_address.clone().into(),
            "250".into(),
        )), // amount2: 250 USD
        12, // trading_fee (12 / 100_000)
    );
    submit_and_wait(&mut amm_tx, client, Some(&lp_wallet), Some(true), Some(true))
        .await
        .expect("AMM setup: AMMCreate failed");

    AmmPool {
        lp_wallet,
        issuer_wallet,
    }
}
