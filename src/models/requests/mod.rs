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
use alloc::borrow::Cow;
pub use book_offers::*;
pub use channel_authorize::*;
pub use channel_verify::*;
pub use deposit_authorize::*;
use derive_new::new;
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
use serde_with::skip_serializing_none;
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

/// The base fields for all request models.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
pub struct CommonFields<'a> {
    /// The request method.
    pub command: RequestMethod,
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
}

impl Request for CommonFields<'_> {
    fn get_command(&self) -> RequestMethod {
        self.command.clone()
    }
}

/// The base trait for all request models.
/// Used to identify the model as a request.
pub trait Request {
    fn get_command(&self) -> RequestMethod;
}
