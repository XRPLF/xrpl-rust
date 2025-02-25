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
pub mod exceptions;
pub mod fee;
pub mod gateway_balances;
pub mod ledger;
pub mod ledger_closed;
pub mod ledger_current;
pub mod ledger_data;
pub mod ledger_entry;
pub mod manifest;
pub mod metadata;
pub mod nft_buy_offers;
pub mod nft_offer;
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

use super::{requests::XRPLRequest, Amount, XRPLModelException, XRPLModelResult};
use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use core::convert::{TryFrom, TryInto};
use exceptions::XRPLResultException;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{value::Index, Map, Value};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NftOffer<'a> {
    pub amount: Amount<'a>,
    pub flags: u32,
    pub nft_offer_index: Cow<'a, str>,
    pub owner: Cow<'a, str>,
    pub destination: Option<Cow<'a, str>>,
    pub expiration: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XRPLOtherResult(Value);

impl TryFrom<XRPLResult<'_>> for XRPLOtherResult {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::Other(value) => Ok(value),
            res => Err(XRPLResultException::UnexpectedResultType(
                "Other".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}

impl From<Value> for XRPLOtherResult {
    fn from(value: Value) -> Self {
        XRPLOtherResult(value)
    }
}

impl From<XRPLOtherResult> for Value {
    fn from(val: XRPLOtherResult) -> Self {
        val.0
    }
}

impl XRPLOtherResult {
    pub fn get(&self, index: impl Index) -> Option<&Value> {
        self.0.get(index)
    }

    pub fn try_get_typed<I, T>(&self, index: I) -> XRPLModelResult<T>
    where
        I: Index,
        T: DeserializeOwned,
    {
        let value = self
            .0
            .get(index)
            .ok_or(XRPLResultException::IndexNotFound)?;

        Ok(serde_json::from_value(value.clone())?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum XRPLResult<'a> {
    AccountChannels(account_channels::AccountChannels<'a>),
    AccountInfo(account_info::AccountInfoVersionMap<'a>),
    AccountCurrencies(account_currencies::AccountCurrencies<'a>),
    AccountLines(account_lines::AccountLines<'a>),
    AccountObjects(account_objects::AccountObjects<'a>),
    AccountNfts(account_nfts::AccountNfts<'a>),
    AccountOffers(account_offers::AccountOffers<'a>),
    AccountTx(account_tx::AccountTxVersionMap<'a>),
    AMMInfo(amm_info::AMMInfo<'a>),
    BookOffers(book_offers::BookOffers<'a>),
    ChannelAuthorize(channel_authorize::ChannelAuthorize<'a>),
    ChannelVerify(channel_verify::ChannelVerify<'a>),
    DepositAuthorized(deposit_authorize::DepositAuthorized<'a>),
    Fee(fee::Fee<'a>),
    GatewayBalances(gateway_balances::GatewayBalances<'a>),
    Ledger(ledger::Ledger<'a>),
    LedgerClosed(ledger_closed::LedgerClosed<'a>),
    LedgerCurrent(ledger_current::LedgerCurrent<'a>),
    LedgerData(ledger_data::LedgerData<'a>),
    LedgerEntry(ledger_entry::LedgerEntry<'a>),
    Manifest(manifest::Manifest<'a>),
    NFTBuyOffers(nft_buy_offers::NFTBuyOffers<'a>),
    NFTSellOffers(nft_sell_offers::NFTSellOffers<'a>),
    NoRippleCheck(no_ripple_check::NoRippleCheck<'a>),
    PathFind(path_find::PathFind<'a>),
    Random(random::Random<'a>),
    RipplePathFind(ripple_path_find::RipplePathFind<'a>),
    ServerInfo(server_info::ServerInfo<'a>),
    ServerState(server_state::ServerState<'a>),
    Submit(submit::Submit<'a>),
    SubmitMultisigned(submit_multisigned::SubmitMultisigned<'a>),
    TransactionEntry(transaction_entry::TransactionEntry<'a>),
    Tx(tx::TxVersionMap<'a>),
    Subscribe(subscribe::Subscribe<'a>),
    Unsubscribe(unsubscribe::Unsubscribe<'a>),
    Ping(ping::Ping<'a>),
    Other(XRPLOtherResult),
}

macro_rules! impl_from_result {
    ($module_name:ident, $variant:ident) => {
        impl<'a> From<$module_name::$variant<'a>> for XRPLResult<'a> {
            fn from(value: $module_name::$variant<'a>) -> Self {
                XRPLResult::$variant(value)
            }
        }
    };
}

impl_from_result!(account_channels, AccountChannels);
impl_from_result!(account_currencies, AccountCurrencies);
impl_from_result!(account_lines, AccountLines);
impl_from_result!(account_objects, AccountObjects);
impl_from_result!(account_nfts, AccountNfts);
impl_from_result!(account_offers, AccountOffers);
impl_from_result!(amm_info, AMMInfo);
impl_from_result!(book_offers, BookOffers);
impl_from_result!(channel_authorize, ChannelAuthorize);
impl_from_result!(channel_verify, ChannelVerify);
impl_from_result!(deposit_authorize, DepositAuthorized);
impl_from_result!(fee, Fee);
impl_from_result!(gateway_balances, GatewayBalances);
impl_from_result!(ledger, Ledger);
impl_from_result!(ledger_closed, LedgerClosed);
impl_from_result!(ledger_current, LedgerCurrent);
impl_from_result!(ledger_data, LedgerData);
impl_from_result!(ledger_entry, LedgerEntry);
impl_from_result!(manifest, Manifest);
impl_from_result!(nft_buy_offers, NFTBuyOffers);
impl_from_result!(nft_sell_offers, NFTSellOffers);
impl_from_result!(no_ripple_check, NoRippleCheck);
impl_from_result!(path_find, PathFind);
impl_from_result!(random, Random);
impl_from_result!(ripple_path_find, RipplePathFind);
impl_from_result!(server_info, ServerInfo);
impl_from_result!(server_state, ServerState);
impl_from_result!(submit, Submit);
impl_from_result!(submit_multisigned, SubmitMultisigned);
impl_from_result!(transaction_entry, TransactionEntry);
impl_from_result!(ping, Ping);
impl_from_result!(subscribe, Subscribe);
impl_from_result!(unsubscribe, Unsubscribe);

impl<'a> From<Value> for XRPLResult<'a> {
    fn from(value: Value) -> Self {
        XRPLResult::Other(XRPLOtherResult(value))
    }
}

impl<'a> From<XRPLOtherResult> for XRPLResult<'a> {
    fn from(other: XRPLOtherResult) -> Self {
        XRPLResult::Other(other)
    }
}

macro_rules! impl_try_from_result {
    ($module_name:ident, $type:ident, $variant:ident) => {
        impl<'a> TryFrom<XRPLResult<'a>> for $module_name::$type<'a> {
            type Error = XRPLModelException;

            fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
                match result {
                    XRPLResult::$variant(value) => Ok(value),
                    res => Err(XRPLResultException::UnexpectedResultType(
                        stringify!($variant).to_string(),
                        res.get_name(),
                    )
                    .into()),
                }
            }
        }
    };
}

impl_try_from_result!(account_channels, AccountChannels, AccountChannels);
impl_try_from_result!(account_currencies, AccountCurrencies, AccountCurrencies);
impl_try_from_result!(account_lines, AccountLines, AccountLines);
impl_try_from_result!(account_objects, AccountObjects, AccountObjects);
impl_try_from_result!(account_nfts, AccountNfts, AccountNfts);
impl_try_from_result!(account_offers, AccountOffers, AccountOffers);
impl_try_from_result!(amm_info, AMMInfo, AMMInfo);
impl_try_from_result!(book_offers, BookOffers, BookOffers);
impl_try_from_result!(channel_authorize, ChannelAuthorize, ChannelAuthorize);
impl_try_from_result!(channel_verify, ChannelVerify, ChannelVerify);
impl_try_from_result!(deposit_authorize, DepositAuthorized, DepositAuthorized);
impl_try_from_result!(fee, Fee, Fee);
impl_try_from_result!(gateway_balances, GatewayBalances, GatewayBalances);
impl_try_from_result!(ledger, Ledger, Ledger);
impl_try_from_result!(ledger_closed, LedgerClosed, LedgerClosed);
impl_try_from_result!(ledger_current, LedgerCurrent, LedgerCurrent);
impl_try_from_result!(ledger_data, LedgerData, LedgerData);
impl_try_from_result!(ledger_entry, LedgerEntry, LedgerEntry);
impl_try_from_result!(manifest, Manifest, Manifest);
impl_try_from_result!(nft_buy_offers, NFTBuyOffers, NFTBuyOffers);
impl_try_from_result!(nft_sell_offers, NFTSellOffers, NFTSellOffers);
impl_try_from_result!(no_ripple_check, NoRippleCheck, NoRippleCheck);
impl_try_from_result!(path_find, PathFind, PathFind);
impl_try_from_result!(random, Random, Random);
impl_try_from_result!(ripple_path_find, RipplePathFind, RipplePathFind);
impl_try_from_result!(server_info, ServerInfo, ServerInfo);
impl_try_from_result!(server_state, ServerState, ServerState);
impl_try_from_result!(submit, Submit, Submit);
impl_try_from_result!(submit_multisigned, SubmitMultisigned, SubmitMultisigned);
impl_try_from_result!(transaction_entry, TransactionEntry, TransactionEntry);
impl_try_from_result!(ping, Ping, Ping);
impl_try_from_result!(subscribe, Subscribe, Subscribe);
impl_try_from_result!(unsubscribe, Unsubscribe, Unsubscribe);

impl<'a> TryInto<Value> for XRPLResult<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<Value> {
        match self {
            XRPLResult::Other(XRPLOtherResult(value)) => Ok(value),
            res => Ok(serde_json::to_value(res)?),
        }
    }
}

impl XRPLResult<'_> {
    pub(crate) fn get_name(&self) -> String {
        match self {
            XRPLResult::AccountChannels(_) => "AccountChannels".to_string(),
            XRPLResult::AccountInfo(_) => "AccountInfo".to_string(),
            XRPLResult::AccountCurrencies(_) => "AccountCurrencies".to_string(),
            XRPLResult::AccountLines(_) => "AccountLines".to_string(),
            XRPLResult::AccountObjects(_) => "AccountObjects".to_string(),
            XRPLResult::AccountNfts(_) => "AccountNfts".to_string(),
            XRPLResult::AccountOffers(_) => "AccountOffers".to_string(),
            XRPLResult::AccountTx(_) => "AccountTx".to_string(),
            XRPLResult::AMMInfo(_) => "AMMInfo".to_string(),
            XRPLResult::BookOffers(_) => "BookOffers".to_string(),
            XRPLResult::ChannelAuthorize(_) => "ChannelAuthorize".to_string(),
            XRPLResult::ChannelVerify(_) => "ChannelVerify".to_string(),
            XRPLResult::DepositAuthorized(_) => "DepositAuthorized".to_string(),
            XRPLResult::Fee(_) => "Fee".to_string(),
            XRPLResult::GatewayBalances(_) => "GatewayBalances".to_string(),
            XRPLResult::Ledger(_) => "Ledger".to_string(),
            XRPLResult::LedgerClosed(_) => "LedgerClosed".to_string(),
            XRPLResult::LedgerCurrent(_) => "LedgerCurrent".to_string(),
            XRPLResult::LedgerData(_) => "LedgerData".to_string(),
            XRPLResult::LedgerEntry(_) => "LedgerEntry".to_string(),
            XRPLResult::Manifest(_) => "Manifest".to_string(),
            XRPLResult::NFTBuyOffers(_) => "NFTBuyOffers".to_string(),
            XRPLResult::NFTSellOffers(_) => "NFTSellOffers".to_string(),
            XRPLResult::NoRippleCheck(_) => "NoRippleCheck".to_string(),
            XRPLResult::PathFind(_) => "PathFind".to_string(),
            XRPLResult::Ping(_) => "Ping".to_string(),
            XRPLResult::Random(_) => "Random".to_string(),
            XRPLResult::RipplePathFind(_) => "RipplePathFind".to_string(),
            XRPLResult::ServerInfo(_) => "ServerInfo".to_string(),
            XRPLResult::ServerState(_) => "ServerState".to_string(),
            XRPLResult::Submit(_) => "Submit".to_string(),
            XRPLResult::SubmitMultisigned(_) => "SubmitMultisigned".to_string(),
            XRPLResult::TransactionEntry(_) => "TransactionEntry".to_string(),
            XRPLResult::Subscribe(_) => "Subscribe".to_string(),
            XRPLResult::Tx(_) => "Tx".to_string(),
            XRPLResult::Unsubscribe(_) => "Unsubscribe".to_string(),
            XRPLResult::Other(_) => "Other".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResponseType {
    Response,
    LedgerClosed,
    Transaction,
}

#[derive(Debug, Clone, Serialize)]
pub struct XRPLResponse<'a> {
    pub id: Option<Cow<'a, str>>,
    pub error: Option<Cow<'a, str>>,
    pub error_code: Option<i32>,
    pub error_message: Option<Cow<'a, str>>,
    pub forwarded: Option<bool>,
    pub request: Option<XRPLRequest<'a>>,
    pub result: Option<XRPLResult<'a>>,
    pub status: Option<ResponseStatus>,
    pub r#type: Option<ResponseType>,
    pub warning: Option<Cow<'a, str>>,
    pub warnings: Option<Cow<'a, [XRPLWarning<'a>]>>,
}

macro_rules! impl_try_from_response {
    ($module_name:ident, $type:ident, $variant:ident) => {
        impl<'a, 'b> TryFrom<XRPLResponse<'a>> for $module_name::$type<'b>
        // Lifetime variance
        where
            'a: 'b,
            'b: 'a,
        {
            type Error = XRPLModelException;

            fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
                match response.result {
                    Some(result) => match result {
                        XRPLResult::$variant(value) => Ok(value),
                        res => Err(XRPLResultException::UnexpectedResultType(
                            stringify!($variant).to_string(),
                            res.get_name(),
                        )
                        .into()),
                    },
                    None => Err(XRPLModelException::MissingField("result".to_string())),
                }
            }
        }
    };
}

impl_try_from_response!(account_channels, AccountChannels, AccountChannels);
impl_try_from_response!(account_currencies, AccountCurrencies, AccountCurrencies);
impl_try_from_response!(account_lines, AccountLines, AccountLines);
impl_try_from_response!(account_objects, AccountObjects, AccountObjects);
impl_try_from_response!(account_nfts, AccountNfts, AccountNfts);
impl_try_from_response!(account_offers, AccountOffers, AccountOffers);
impl_try_from_response!(amm_info, AMMInfo, AMMInfo);
impl_try_from_response!(book_offers, BookOffers, BookOffers);
impl_try_from_response!(channel_authorize, ChannelAuthorize, ChannelAuthorize);
impl_try_from_response!(channel_verify, ChannelVerify, ChannelVerify);
impl_try_from_response!(deposit_authorize, DepositAuthorized, DepositAuthorized);
impl_try_from_response!(fee, Fee, Fee);
impl_try_from_response!(gateway_balances, GatewayBalances, GatewayBalances);
impl_try_from_response!(ledger, Ledger, Ledger);
impl_try_from_response!(ledger_closed, LedgerClosed, LedgerClosed);
impl_try_from_response!(ledger_current, LedgerCurrent, LedgerCurrent);
impl_try_from_response!(ledger_data, LedgerData, LedgerData);
impl_try_from_response!(ledger_entry, LedgerEntry, LedgerEntry);
impl_try_from_response!(manifest, Manifest, Manifest);
impl_try_from_response!(nft_buy_offers, NFTBuyOffers, NFTBuyOffers);
impl_try_from_response!(nft_sell_offers, NFTSellOffers, NFTSellOffers);
impl_try_from_response!(no_ripple_check, NoRippleCheck, NoRippleCheck);
impl_try_from_response!(path_find, PathFind, PathFind);
impl_try_from_response!(ping, Ping, Ping);
impl_try_from_response!(random, Random, Random);
impl_try_from_response!(ripple_path_find, RipplePathFind, RipplePathFind);
impl_try_from_response!(server_info, ServerInfo, ServerInfo);
impl_try_from_response!(server_state, ServerState, ServerState);
impl_try_from_response!(submit, Submit, Submit);
impl_try_from_response!(submit_multisigned, SubmitMultisigned, SubmitMultisigned);
impl_try_from_response!(transaction_entry, TransactionEntry, TransactionEntry);
impl_try_from_response!(subscribe, Subscribe, Subscribe);
impl_try_from_response!(unsubscribe, Unsubscribe, Unsubscribe);

fn is_subscription_stream_item(item: &Map<String, Value>) -> bool {
    item.get("result").is_none() && item.get("error_code").is_none()
}

impl<'a, 'de> Deserialize<'de> for XRPLResponse<'a> {
    fn deserialize<D>(deserializer: D) -> XRPLModelResult<XRPLResponse<'a>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: add validation for fields that can not coexist in the same response
        let mut map = serde_json::Map::deserialize(deserializer)?;
        if map.is_empty() {
            return Err(serde::de::Error::custom("Empty response"));
        }
        if is_subscription_stream_item(&map) {
            let map_as_value = Value::Object(map);
            Ok(XRPLResponse {
                id: None,
                error: None,
                error_code: None,
                error_message: None,
                forwarded: None,
                request: None,
                result: serde_json::from_value(map_as_value).map_err(serde::de::Error::custom)?,
                status: None,
                r#type: None,
                warning: None,
                warnings: None,
            })
        } else {
            Ok(XRPLResponse {
                id: map
                    .remove("id")
                    .and_then(|v| serde_json::from_value(v).ok()),
                error: map
                    .remove("error")
                    .and_then(|v| serde_json::from_value(v).ok()),
                error_code: map
                    .remove("error_code")
                    .and_then(|v| serde_json::from_value(v).ok()),
                error_message: map
                    .remove("error_message")
                    .and_then(|v| serde_json::from_value(v).ok()),
                forwarded: map.remove("forwarded").and_then(|v| v.as_bool()),
                request: map
                    .remove("request")
                    .and_then(|v| serde_json::from_value(v).ok()),
                result: map
                    .remove("result")
                    .and_then(|v| serde_json::from_value(v).ok()),
                status: map
                    .remove("status")
                    .and_then(|v| serde_json::from_value(v).ok()),
                r#type: map
                    .remove("type")
                    .and_then(|v| serde_json::from_value(v).ok()),
                warning: map
                    .remove("warning")
                    .and_then(|v| serde_json::from_value(v).ok()),
                warnings: map
                    .remove("warnings")
                    .and_then(|v| serde_json::from_value(v).ok()),
            })
        }
    }
}

impl<'a> TryInto<XRPLResult<'a>> for XRPLResponse<'a> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<XRPLResult<'a>> {
        if self.is_success() {
            if let Some(result) = self.result {
                Ok(result)
            } else {
                Err(XRPLResultException::ExpectedResultOrError.into())
            }
        } else {
            Err(XRPLResultException::ResponseError(
                self.error_message
                    .unwrap_or(self.error.unwrap_or_else(|| "Unknown error".into()))
                    .to_string(),
            )
            .into())
        }
    }
}

impl<'a> XRPLResponse<'a> {
    pub fn is_success(&self) -> bool {
        if let Some(status) = &self.status {
            status == &ResponseStatus::Success
        } else if let Some(result) = &self.result {
            match serde_json::to_value(result) {
                Ok(value) => match value.get("status") {
                    Some(Value::String(status)) => status == "success",
                    _ => false,
                },
                _ => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPLWarning<'a> {
    pub id: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub forwarded: Option<bool>,
}
