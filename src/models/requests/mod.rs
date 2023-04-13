pub mod account_channels;
pub mod account_currencies;
pub mod account_info;
pub mod account_lines;
pub mod account_nfts;
pub mod account_objects;
pub mod account_offers;
pub mod account_tx;
pub mod book_offers;
pub mod channel_authorize;
pub mod channel_verify;
pub mod deposit_authorize;
pub mod exceptions;
pub mod fee;
pub mod gateway_balances;
pub mod ledger;
pub mod ledger_closed;
pub mod ledger_current;
pub mod ledger_data;
pub mod ledger_entry;
pub mod manifest;
pub mod nft_buy_offers;
pub mod nft_sell_offers;
pub mod no_ripple_check;
pub mod path_find;
pub mod ping;
pub mod random;
pub mod ripple_path_find;
pub mod server_info;
pub mod server_state;
pub mod submit;
pub mod submit_multisigned;
pub mod subscribe;
pub mod transaction_entry;
pub mod tx;
pub mod unsubscribe;

pub use account_channels::*;
pub use account_currencies::*;
pub use account_info::*;
pub use account_lines::*;
pub use account_nfts::*;
pub use account_objects::*;
pub use account_offers::*;
pub use account_tx::*;
pub use book_offers::*;
pub use channel_authorize::*;
pub use channel_verify::*;
pub use deposit_authorize::*;
pub use exceptions::*;
pub use fee::*;
pub use gateway_balances::*;
pub use ledger::*;
pub use ledger_closed::*;
pub use ledger_current::*;
pub use ledger_data::*;
pub use ledger_entry::*;
pub use manifest::*;
pub use nft_buy_offers::*;
pub use nft_sell_offers::*;
pub use no_ripple_check::*;
pub use path_find::*;
pub use ping::*;
pub use random::*;
pub use ripple_path_find::*;
pub use server_info::*;
pub use server_state::*;
pub use submit::*;
pub use submit_multisigned::*;
pub use subscribe::*;
pub use transaction_entry::*;
pub use tx::*;
pub use unsubscribe::*;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Represents the different options for the `method`
/// field in a request.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RequestMethod {
    // Account methods
    AccountChannels,
    AccountCurrencies,
    AccountInfo,
    AccountLines,
    AccountNfts,
    AccountObjects,
    AccountOffers,
    AccountTx,
    GatewayBalances,
    NoRippleCheck,

    // Transaction methods
    Sign,
    SignFor,
    Submit,
    SubmitMultisigned,
    TransactionEntry,
    Tx,

    // Channel methods
    ChannelAuthorize,
    ChannelVerify,

    // Path methods
    BookOffers,
    DepositAuthorized,
    NftBuyOffers,
    NftSellOffers,
    PathFind,
    RipplePathFind,

    // Ledger methods
    Ledger,
    LedgerClosed,
    LedgerCurrent,
    LedgerData,
    LedgerEntry,

    // Subscription methods
    Subscribe,
    Unsubscribe,

    // Server info methods
    Fee,
    Manifest,
    ServerInfo,
    ServerState,

    // Utility methods
    Ping,
    Random,
}

/// For use with serde defaults.
/// TODO Find a better way
impl RequestMethod {
    fn account_channels() -> Self {
        RequestMethod::AccountChannels
    }
    fn account_currencies() -> Self {
        RequestMethod::AccountCurrencies
    }
    fn account_info() -> Self {
        RequestMethod::AccountInfo
    }
    fn account_lines() -> Self {
        RequestMethod::AccountLines
    }
    fn account_nfts() -> Self {
        RequestMethod::AccountNfts
    }
    fn account_objects() -> Self {
        RequestMethod::AccountObjects
    }
    fn account_offers() -> Self {
        RequestMethod::AccountOffers
    }
    fn account_tx() -> Self {
        RequestMethod::AccountTx
    }
    fn book_offers() -> Self {
        RequestMethod::BookOffers
    }
    fn channel_authorize() -> Self {
        RequestMethod::ChannelAuthorize
    }
    fn channel_verify() -> Self {
        RequestMethod::ChannelVerify
    }
    fn deposit_authorization() -> Self {
        RequestMethod::DepositAuthorized
    }
    fn fee() -> Self {
        RequestMethod::Fee
    }
    fn ledger_closed() -> Self {
        RequestMethod::LedgerClosed
    }
    fn ledger_current() -> Self {
        RequestMethod::LedgerCurrent
    }
    fn ledger_data() -> Self {
        RequestMethod::LedgerData
    }
    fn ledger_entry() -> Self {
        RequestMethod::LedgerEntry
    }
    fn ledger() -> Self {
        RequestMethod::Ledger
    }
    fn manifest() -> Self {
        RequestMethod::Manifest
    }
    fn nft_buy_offers() -> Self {
        RequestMethod::NftBuyOffers
    }
    fn nft_sell_offers() -> Self {
        RequestMethod::NftSellOffers
    }
    fn no_ripple_check() -> Self {
        RequestMethod::NoRippleCheck
    }
    fn path_find() -> Self {
        RequestMethod::PathFind
    }
    fn ripple_path_find() -> Self {
        RequestMethod::RipplePathFind
    }
    fn ping() -> Self {
        RequestMethod::Ping
    }
    fn random() -> Self {
        RequestMethod::Random
    }
    fn server_info() -> Self {
        RequestMethod::ServerInfo
    }
    fn server_state() -> Self {
        RequestMethod::ServerState
    }
    fn submit() -> Self {
        RequestMethod::Submit
    }
    fn sign_for() -> Self {
        RequestMethod::SignFor
    }
    fn sign() -> Self {
        RequestMethod::Sign
    }
    fn submit_multisigned() -> Self {
        RequestMethod::SubmitMultisigned
    }
    fn subscribe() -> Self {
        RequestMethod::Subscribe
    }
    fn unsubscribe() -> Self {
        RequestMethod::Unsubscribe
    }
    fn transaction_entry() -> Self {
        RequestMethod::TransactionEntry
    }
    fn tx() -> Self {
        RequestMethod::Tx
    }
}
