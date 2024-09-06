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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum XRPLRequest<'a> {
    AccountChannels(AccountChannels<'a>),
    AccountCurrencies(AccountCurrencies<'a>),
    AccountInfo(AccountInfo<'a>),
    AccountLines(AccountLines<'a>),
    AccountNfts(AccountNfts<'a>),
    AccountObjects(AccountObjects<'a>),
    AccountOffers(AccountOffers<'a>),
    AccountTx(AccountTx<'a>),
    GatewayBalances(GatewayBalances<'a>),
    NoRippleCheck(NoRippleCheck<'a>),
    Submit(Submit<'a>),
    SubmitMultisigned(SubmitMultisigned<'a>),
    TransactionEntry(TransactionEntry<'a>),
    Tx(Tx<'a>),
    ChannelAuthorize(ChannelAuthorize<'a>),
    ChannelVerify(ChannelVerify<'a>),
    BookOffers(BookOffers<'a>),
    DepositAuthorized(DepositAuthorized<'a>),
    NftBuyOffers(NftBuyOffers<'a>),
    NftSellOffers(NftSellOffers<'a>),
    PathFind(PathFind<'a>),
    RipplePathFind(RipplePathFind<'a>),
    Ledger(Ledger<'a>),
    LedgerClosed(LedgerClosed<'a>),
    LedgerCurrent(LedgerCurrent<'a>),
    LedgerData(LedgerData<'a>),
    LedgerEntry(LedgerEntry<'a>),
    Subscribe(Subscribe<'a>),
    Unsubscribe(Unsubscribe<'a>),
    Fee(Fee<'a>),
    Manifest(Manifest<'a>),
    ServerInfo(ServerInfo<'a>),
    ServerState(ServerState<'a>),
    Ping(Ping<'a>),
    Random(Random<'a>),
}

impl<'a> From<AccountChannels<'a>> for XRPLRequest<'a> {
    fn from(request: AccountChannels<'a>) -> Self {
        XRPLRequest::AccountChannels(request)
    }
}

impl<'a> From<AccountCurrencies<'a>> for XRPLRequest<'a> {
    fn from(request: AccountCurrencies<'a>) -> Self {
        XRPLRequest::AccountCurrencies(request)
    }
}

impl<'a> From<AccountInfo<'a>> for XRPLRequest<'a> {
    fn from(request: AccountInfo<'a>) -> Self {
        XRPLRequest::AccountInfo(request)
    }
}

impl<'a> From<AccountLines<'a>> for XRPLRequest<'a> {
    fn from(request: AccountLines<'a>) -> Self {
        XRPLRequest::AccountLines(request)
    }
}

impl<'a> From<AccountNfts<'a>> for XRPLRequest<'a> {
    fn from(request: AccountNfts<'a>) -> Self {
        XRPLRequest::AccountNfts(request)
    }
}

impl<'a> From<AccountObjects<'a>> for XRPLRequest<'a> {
    fn from(request: AccountObjects<'a>) -> Self {
        XRPLRequest::AccountObjects(request)
    }
}

impl<'a> From<AccountOffers<'a>> for XRPLRequest<'a> {
    fn from(request: AccountOffers<'a>) -> Self {
        XRPLRequest::AccountOffers(request)
    }
}

impl<'a> From<AccountTx<'a>> for XRPLRequest<'a> {
    fn from(request: AccountTx<'a>) -> Self {
        XRPLRequest::AccountTx(request)
    }
}

impl<'a> From<GatewayBalances<'a>> for XRPLRequest<'a> {
    fn from(request: GatewayBalances<'a>) -> Self {
        XRPLRequest::GatewayBalances(request)
    }
}

impl<'a> From<NoRippleCheck<'a>> for XRPLRequest<'a> {
    fn from(request: NoRippleCheck<'a>) -> Self {
        XRPLRequest::NoRippleCheck(request)
    }
}

impl<'a> From<Submit<'a>> for XRPLRequest<'a> {
    fn from(request: Submit<'a>) -> Self {
        XRPLRequest::Submit(request)
    }
}

impl<'a> From<SubmitMultisigned<'a>> for XRPLRequest<'a> {
    fn from(request: SubmitMultisigned<'a>) -> Self {
        XRPLRequest::SubmitMultisigned(request)
    }
}

impl<'a> From<TransactionEntry<'a>> for XRPLRequest<'a> {
    fn from(request: TransactionEntry<'a>) -> Self {
        XRPLRequest::TransactionEntry(request)
    }
}

impl<'a> From<Tx<'a>> for XRPLRequest<'a> {
    fn from(request: Tx<'a>) -> Self {
        XRPLRequest::Tx(request)
    }
}

impl<'a> From<ChannelAuthorize<'a>> for XRPLRequest<'a> {
    fn from(request: ChannelAuthorize<'a>) -> Self {
        XRPLRequest::ChannelAuthorize(request)
    }
}

impl<'a> From<ChannelVerify<'a>> for XRPLRequest<'a> {
    fn from(request: ChannelVerify<'a>) -> Self {
        XRPLRequest::ChannelVerify(request)
    }
}

impl<'a> From<BookOffers<'a>> for XRPLRequest<'a> {
    fn from(request: BookOffers<'a>) -> Self {
        XRPLRequest::BookOffers(request)
    }
}

impl<'a> From<DepositAuthorized<'a>> for XRPLRequest<'a> {
    fn from(request: DepositAuthorized<'a>) -> Self {
        XRPLRequest::DepositAuthorized(request)
    }
}

impl<'a> From<NftBuyOffers<'a>> for XRPLRequest<'a> {
    fn from(request: NftBuyOffers<'a>) -> Self {
        XRPLRequest::NftBuyOffers(request)
    }
}

impl<'a> From<NftSellOffers<'a>> for XRPLRequest<'a> {
    fn from(request: NftSellOffers<'a>) -> Self {
        XRPLRequest::NftSellOffers(request)
    }
}

impl<'a> From<PathFind<'a>> for XRPLRequest<'a> {
    fn from(request: PathFind<'a>) -> Self {
        XRPLRequest::PathFind(request)
    }
}

impl<'a> From<RipplePathFind<'a>> for XRPLRequest<'a> {
    fn from(request: RipplePathFind<'a>) -> Self {
        XRPLRequest::RipplePathFind(request)
    }
}

impl<'a> From<Ledger<'a>> for XRPLRequest<'a> {
    fn from(request: Ledger<'a>) -> Self {
        XRPLRequest::Ledger(request)
    }
}

impl<'a> From<LedgerClosed<'a>> for XRPLRequest<'a> {
    fn from(request: LedgerClosed<'a>) -> Self {
        XRPLRequest::LedgerClosed(request)
    }
}

impl<'a> From<LedgerCurrent<'a>> for XRPLRequest<'a> {
    fn from(request: LedgerCurrent<'a>) -> Self {
        XRPLRequest::LedgerCurrent(request)
    }
}

impl<'a> From<LedgerData<'a>> for XRPLRequest<'a> {
    fn from(request: LedgerData<'a>) -> Self {
        XRPLRequest::LedgerData(request)
    }
}

impl<'a> From<LedgerEntry<'a>> for XRPLRequest<'a> {
    fn from(request: LedgerEntry<'a>) -> Self {
        XRPLRequest::LedgerEntry(request)
    }
}

impl<'a> From<Subscribe<'a>> for XRPLRequest<'a> {
    fn from(request: Subscribe<'a>) -> Self {
        XRPLRequest::Subscribe(request)
    }
}

impl<'a> From<Unsubscribe<'a>> for XRPLRequest<'a> {
    fn from(request: Unsubscribe<'a>) -> Self {
        XRPLRequest::Unsubscribe(request)
    }
}

impl<'a> From<Fee<'a>> for XRPLRequest<'a> {
    fn from(request: Fee<'a>) -> Self {
        XRPLRequest::Fee(request)
    }
}

impl<'a> From<Manifest<'a>> for XRPLRequest<'a> {
    fn from(request: Manifest<'a>) -> Self {
        XRPLRequest::Manifest(request)
    }
}

impl<'a> From<ServerInfo<'a>> for XRPLRequest<'a> {
    fn from(request: ServerInfo<'a>) -> Self {
        XRPLRequest::ServerInfo(request)
    }
}

impl<'a> From<ServerState<'a>> for XRPLRequest<'a> {
    fn from(request: ServerState<'a>) -> Self {
        XRPLRequest::ServerState(request)
    }
}

impl<'a> From<Ping<'a>> for XRPLRequest<'a> {
    fn from(request: Ping<'a>) -> Self {
        XRPLRequest::Ping(request)
    }
}

impl<'a> From<Random<'a>> for XRPLRequest<'a> {
    fn from(request: Random<'a>) -> Self {
        XRPLRequest::Random(request)
    }
}

impl<'a> Request<'a> for XRPLRequest<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        match self {
            XRPLRequest::AccountChannels(request) => request.get_common_fields(),
            XRPLRequest::AccountCurrencies(request) => request.get_common_fields(),
            XRPLRequest::AccountInfo(request) => request.get_common_fields(),
            XRPLRequest::AccountLines(request) => request.get_common_fields(),
            XRPLRequest::AccountNfts(request) => request.get_common_fields(),
            XRPLRequest::AccountObjects(request) => request.get_common_fields(),
            XRPLRequest::AccountOffers(request) => request.get_common_fields(),
            XRPLRequest::AccountTx(request) => request.get_common_fields(),
            XRPLRequest::GatewayBalances(request) => request.get_common_fields(),
            XRPLRequest::NoRippleCheck(request) => request.get_common_fields(),
            XRPLRequest::Submit(request) => request.get_common_fields(),
            XRPLRequest::SubmitMultisigned(request) => request.get_common_fields(),
            XRPLRequest::TransactionEntry(request) => request.get_common_fields(),
            XRPLRequest::Tx(request) => request.get_common_fields(),
            XRPLRequest::ChannelAuthorize(request) => request.get_common_fields(),
            XRPLRequest::ChannelVerify(request) => request.get_common_fields(),
            XRPLRequest::BookOffers(request) => request.get_common_fields(),
            XRPLRequest::DepositAuthorized(request) => request.get_common_fields(),
            XRPLRequest::NftBuyOffers(request) => request.get_common_fields(),
            XRPLRequest::NftSellOffers(request) => request.get_common_fields(),
            XRPLRequest::PathFind(request) => request.get_common_fields(),
            XRPLRequest::RipplePathFind(request) => request.get_common_fields(),
            XRPLRequest::Ledger(request) => request.get_common_fields(),
            XRPLRequest::LedgerClosed(request) => request.get_common_fields(),
            XRPLRequest::LedgerCurrent(request) => request.get_common_fields(),
            XRPLRequest::LedgerData(request) => request.get_common_fields(),
            XRPLRequest::LedgerEntry(request) => request.get_common_fields(),
            XRPLRequest::Subscribe(request) => request.get_common_fields(),
            XRPLRequest::Unsubscribe(request) => request.get_common_fields(),
            XRPLRequest::Fee(request) => request.get_common_fields(),
            XRPLRequest::Manifest(request) => request.get_common_fields(),
            XRPLRequest::ServerInfo(request) => request.get_common_fields(),
            XRPLRequest::ServerState(request) => request.get_common_fields(),
            XRPLRequest::Ping(request) => request.get_common_fields(),
            XRPLRequest::Random(request) => request.get_common_fields(),
        }
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        match self {
            XRPLRequest::AccountChannels(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountCurrencies(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountInfo(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountLines(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountNfts(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountObjects(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountOffers(request) => request.get_common_fields_mut(),
            XRPLRequest::AccountTx(request) => request.get_common_fields_mut(),
            XRPLRequest::GatewayBalances(request) => request.get_common_fields_mut(),
            XRPLRequest::NoRippleCheck(request) => request.get_common_fields_mut(),
            XRPLRequest::Submit(request) => request.get_common_fields_mut(),
            XRPLRequest::SubmitMultisigned(request) => request.get_common_fields_mut(),
            XRPLRequest::TransactionEntry(request) => request.get_common_fields_mut(),
            XRPLRequest::Tx(request) => request.get_common_fields_mut(),
            XRPLRequest::ChannelAuthorize(request) => request.get_common_fields_mut(),
            XRPLRequest::ChannelVerify(request) => request.get_common_fields_mut(),
            XRPLRequest::BookOffers(request) => request.get_common_fields_mut(),
            XRPLRequest::DepositAuthorized(request) => request.get_common_fields_mut(),
            XRPLRequest::NftBuyOffers(request) => request.get_common_fields_mut(),
            XRPLRequest::NftSellOffers(request) => request.get_common_fields_mut(),
            XRPLRequest::PathFind(request) => request.get_common_fields_mut(),
            XRPLRequest::RipplePathFind(request) => request.get_common_fields_mut(),
            XRPLRequest::Ledger(request) => request.get_common_fields_mut(),
            XRPLRequest::LedgerClosed(request) => request.get_common_fields_mut(),
            XRPLRequest::LedgerCurrent(request) => request.get_common_fields_mut(),
            XRPLRequest::LedgerData(request) => request.get_common_fields_mut(),
            XRPLRequest::LedgerEntry(request) => request.get_common_fields_mut(),
            XRPLRequest::Subscribe(request) => request.get_common_fields_mut(),
            XRPLRequest::Unsubscribe(request) => request.get_common_fields_mut(),
            XRPLRequest::Fee(request) => request.get_common_fields_mut(),
            XRPLRequest::Manifest(request) => request.get_common_fields_mut(),
            XRPLRequest::ServerInfo(request) => request.get_common_fields_mut(),
            XRPLRequest::ServerState(request) => request.get_common_fields_mut(),
            XRPLRequest::Ping(request) => request.get_common_fields_mut(),
            XRPLRequest::Random(request) => request.get_common_fields_mut(),
        }
    }
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

/// The base trait for all request models.
/// Used to identify the model as a request.
pub trait Request<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a>;
    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct FundFaucet<'a> {
    pub destination: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_context: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<Cow<'a, str>>,
}
