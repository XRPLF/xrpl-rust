pub mod account_channels;
pub mod account_currencies;
pub mod account_info;
pub mod account_lines;
pub mod account_nfts;
pub mod account_objects;
pub mod account_offers;
pub mod account_tx;
pub mod amm_info;
pub mod book_offers;
pub mod channel_authorize;
pub mod channel_verify;
pub mod deposit_authorize;
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

use alloc::borrow::Cow;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
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
    #[serde(rename = "amm_info")]
    AMMInfo,
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
    AccountChannels(account_channels::AccountChannels<'a>),
    AccountCurrencies(account_currencies::AccountCurrencies<'a>),
    AccountInfo(account_info::AccountInfo<'a>),
    AccountLines(account_lines::AccountLines<'a>),
    AccountNfts(account_nfts::AccountNfts<'a>),
    AccountObjects(account_objects::AccountObjects<'a>),
    AccountOffers(account_offers::AccountOffers<'a>),
    AccountTx(account_tx::AccountTx<'a>),
    AMMInfo(amm_info::AMMInfo<'a>),
    GatewayBalances(gateway_balances::GatewayBalances<'a>),
    NoRippleCheck(no_ripple_check::NoRippleCheck<'a>),
    Submit(submit::Submit<'a>),
    SubmitMultisigned(submit_multisigned::SubmitMultisigned<'a>),
    TransactionEntry(transaction_entry::TransactionEntry<'a>),
    Tx(tx::Tx<'a>),
    ChannelAuthorize(channel_authorize::ChannelAuthorize<'a>),
    ChannelVerify(channel_verify::ChannelVerify<'a>),
    BookOffers(book_offers::BookOffers<'a>),
    DepositAuthorized(deposit_authorize::DepositAuthorized<'a>),
    NftBuyOffers(nft_buy_offers::NftBuyOffers<'a>),
    NftSellOffers(nft_sell_offers::NftSellOffers<'a>),
    PathFind(path_find::PathFind<'a>),
    RipplePathFind(ripple_path_find::RipplePathFind<'a>),
    Ledger(ledger::Ledger<'a>),
    LedgerClosed(ledger_closed::LedgerClosed<'a>),
    LedgerCurrent(ledger_current::LedgerCurrent<'a>),
    LedgerData(ledger_data::LedgerData<'a>),
    LedgerEntry(ledger_entry::LedgerEntry<'a>),
    Subscribe(subscribe::Subscribe<'a>),
    Unsubscribe(unsubscribe::Unsubscribe<'a>),
    Fee(fee::Fee<'a>),
    Manifest(manifest::Manifest<'a>),
    ServerInfo(server_info::ServerInfo<'a>),
    ServerState(server_state::ServerState<'a>),
    Ping(ping::Ping<'a>),
    Random(random::Random<'a>),
}

impl<'a> From<account_channels::AccountChannels<'a>> for XRPLRequest<'a> {
    fn from(request: account_channels::AccountChannels<'a>) -> Self {
        XRPLRequest::AccountChannels(request)
    }
}

impl<'a> From<account_currencies::AccountCurrencies<'a>> for XRPLRequest<'a> {
    fn from(request: account_currencies::AccountCurrencies<'a>) -> Self {
        XRPLRequest::AccountCurrencies(request)
    }
}

impl<'a> From<account_info::AccountInfo<'a>> for XRPLRequest<'a> {
    fn from(request: account_info::AccountInfo<'a>) -> Self {
        XRPLRequest::AccountInfo(request)
    }
}

impl<'a> From<account_lines::AccountLines<'a>> for XRPLRequest<'a> {
    fn from(request: account_lines::AccountLines<'a>) -> Self {
        XRPLRequest::AccountLines(request)
    }
}

impl<'a> From<account_nfts::AccountNfts<'a>> for XRPLRequest<'a> {
    fn from(request: account_nfts::AccountNfts<'a>) -> Self {
        XRPLRequest::AccountNfts(request)
    }
}

impl<'a> From<account_objects::AccountObjects<'a>> for XRPLRequest<'a> {
    fn from(request: account_objects::AccountObjects<'a>) -> Self {
        XRPLRequest::AccountObjects(request)
    }
}

impl<'a> From<account_offers::AccountOffers<'a>> for XRPLRequest<'a> {
    fn from(request: account_offers::AccountOffers<'a>) -> Self {
        XRPLRequest::AccountOffers(request)
    }
}

impl<'a> From<account_tx::AccountTx<'a>> for XRPLRequest<'a> {
    fn from(request: account_tx::AccountTx<'a>) -> Self {
        XRPLRequest::AccountTx(request)
    }
}

impl<'a> From<amm_info::AMMInfo<'a>> for XRPLRequest<'a> {
    fn from(request: amm_info::AMMInfo<'a>) -> Self {
        XRPLRequest::AMMInfo(request)
    }
}

impl<'a> From<gateway_balances::GatewayBalances<'a>> for XRPLRequest<'a> {
    fn from(request: gateway_balances::GatewayBalances<'a>) -> Self {
        XRPLRequest::GatewayBalances(request)
    }
}

impl<'a> From<no_ripple_check::NoRippleCheck<'a>> for XRPLRequest<'a> {
    fn from(request: no_ripple_check::NoRippleCheck<'a>) -> Self {
        XRPLRequest::NoRippleCheck(request)
    }
}

impl<'a> From<submit::Submit<'a>> for XRPLRequest<'a> {
    fn from(request: submit::Submit<'a>) -> Self {
        XRPLRequest::Submit(request)
    }
}

impl<'a> From<submit_multisigned::SubmitMultisigned<'a>> for XRPLRequest<'a> {
    fn from(request: submit_multisigned::SubmitMultisigned<'a>) -> Self {
        XRPLRequest::SubmitMultisigned(request)
    }
}

impl<'a> From<transaction_entry::TransactionEntry<'a>> for XRPLRequest<'a> {
    fn from(request: transaction_entry::TransactionEntry<'a>) -> Self {
        XRPLRequest::TransactionEntry(request)
    }
}

impl<'a> From<tx::Tx<'a>> for XRPLRequest<'a> {
    fn from(request: tx::Tx<'a>) -> Self {
        XRPLRequest::Tx(request)
    }
}

impl<'a> From<channel_authorize::ChannelAuthorize<'a>> for XRPLRequest<'a> {
    fn from(request: channel_authorize::ChannelAuthorize<'a>) -> Self {
        XRPLRequest::ChannelAuthorize(request)
    }
}

impl<'a> From<channel_verify::ChannelVerify<'a>> for XRPLRequest<'a> {
    fn from(request: channel_verify::ChannelVerify<'a>) -> Self {
        XRPLRequest::ChannelVerify(request)
    }
}

impl<'a> From<book_offers::BookOffers<'a>> for XRPLRequest<'a> {
    fn from(request: book_offers::BookOffers<'a>) -> Self {
        XRPLRequest::BookOffers(request)
    }
}

impl<'a> From<deposit_authorize::DepositAuthorized<'a>> for XRPLRequest<'a> {
    fn from(request: deposit_authorize::DepositAuthorized<'a>) -> Self {
        XRPLRequest::DepositAuthorized(request)
    }
}

impl<'a> From<nft_buy_offers::NftBuyOffers<'a>> for XRPLRequest<'a> {
    fn from(request: nft_buy_offers::NftBuyOffers<'a>) -> Self {
        XRPLRequest::NftBuyOffers(request)
    }
}

impl<'a> From<nft_sell_offers::NftSellOffers<'a>> for XRPLRequest<'a> {
    fn from(request: nft_sell_offers::NftSellOffers<'a>) -> Self {
        XRPLRequest::NftSellOffers(request)
    }
}

impl<'a> From<path_find::PathFind<'a>> for XRPLRequest<'a> {
    fn from(request: path_find::PathFind<'a>) -> Self {
        XRPLRequest::PathFind(request)
    }
}

impl<'a> From<ripple_path_find::RipplePathFind<'a>> for XRPLRequest<'a> {
    fn from(request: ripple_path_find::RipplePathFind<'a>) -> Self {
        XRPLRequest::RipplePathFind(request)
    }
}

impl<'a> From<ledger::Ledger<'a>> for XRPLRequest<'a> {
    fn from(request: ledger::Ledger<'a>) -> Self {
        XRPLRequest::Ledger(request)
    }
}

impl<'a> From<ledger_closed::LedgerClosed<'a>> for XRPLRequest<'a> {
    fn from(request: ledger_closed::LedgerClosed<'a>) -> Self {
        XRPLRequest::LedgerClosed(request)
    }
}

impl<'a> From<ledger_current::LedgerCurrent<'a>> for XRPLRequest<'a> {
    fn from(request: ledger_current::LedgerCurrent<'a>) -> Self {
        XRPLRequest::LedgerCurrent(request)
    }
}

impl<'a> From<ledger_data::LedgerData<'a>> for XRPLRequest<'a> {
    fn from(request: ledger_data::LedgerData<'a>) -> Self {
        XRPLRequest::LedgerData(request)
    }
}

impl<'a> From<ledger_entry::LedgerEntry<'a>> for XRPLRequest<'a> {
    fn from(request: ledger_entry::LedgerEntry<'a>) -> Self {
        XRPLRequest::LedgerEntry(request)
    }
}

impl<'a> From<subscribe::Subscribe<'a>> for XRPLRequest<'a> {
    fn from(request: subscribe::Subscribe<'a>) -> Self {
        XRPLRequest::Subscribe(request)
    }
}

impl<'a> From<unsubscribe::Unsubscribe<'a>> for XRPLRequest<'a> {
    fn from(request: unsubscribe::Unsubscribe<'a>) -> Self {
        XRPLRequest::Unsubscribe(request)
    }
}

impl<'a> From<fee::Fee<'a>> for XRPLRequest<'a> {
    fn from(request: fee::Fee<'a>) -> Self {
        XRPLRequest::Fee(request)
    }
}

impl<'a> From<manifest::Manifest<'a>> for XRPLRequest<'a> {
    fn from(request: manifest::Manifest<'a>) -> Self {
        XRPLRequest::Manifest(request)
    }
}

impl<'a> From<server_info::ServerInfo<'a>> for XRPLRequest<'a> {
    fn from(request: server_info::ServerInfo<'a>) -> Self {
        XRPLRequest::ServerInfo(request)
    }
}

impl<'a> From<server_state::ServerState<'a>> for XRPLRequest<'a> {
    fn from(request: server_state::ServerState<'a>) -> Self {
        XRPLRequest::ServerState(request)
    }
}

impl<'a> From<ping::Ping<'a>> for XRPLRequest<'a> {
    fn from(request: ping::Ping<'a>) -> Self {
        XRPLRequest::Ping(request)
    }
}

impl<'a> From<random::Random<'a>> for XRPLRequest<'a> {
    fn from(request: random::Random<'a>) -> Self {
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
            XRPLRequest::AMMInfo(request) => request.get_common_fields(),
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
            XRPLRequest::AMMInfo(request) => request.get_common_fields_mut(),
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundFaucet<'a> {
    pub destination: Cow<'a, str>,
    pub usage_context: Option<Cow<'a, str>>,
    pub user_agent: Option<Cow<'a, str>>,
}
