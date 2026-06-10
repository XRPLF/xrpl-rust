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
pub mod nft_info;
pub mod nft_offer;
pub mod nft_sell_offers;
pub mod nftoken;
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
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
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
    NFTInfo(nft_info::NFTInfo<'a>),
    NFTBuyOffers(nft_buy_offers::NFTBuyOffers<'a>),
    NFTSellOffers(nft_sell_offers::NFTSellOffers<'a>),
    NFTokenMintResult(nftoken::NFTokenMintResult<'a>),
    NoRippleCheck(no_ripple_check::NoRippleCheck<'a>),
    PathFind(path_find::PathFind<'a>),
    Random(random::Random<'a>),
    RipplePathFind(ripple_path_find::RipplePathFind<'a>),
    ServerInfo(Box<server_info::ServerInfo<'a>>),
    ServerState(Box<server_state::ServerState<'a>>),
    Submit(submit::Submit<'a>),
    SubmitMultisigned(submit_multisigned::SubmitMultisigned<'a>),
    TransactionEntry(transaction_entry::TransactionEntry<'a>),
    Tx(tx::TxVersionMap<'a>),
    // Other must come before Subscribe/Unsubscribe/Ping so that unrecognized
    // JSON objects fall into Other(Value) — where the raw data is recoverable —
    // rather than into Subscribe (empty PhantomData struct, data unrecoverable).
    Other(XRPLOtherResult),
    Subscribe(subscribe::Subscribe<'a>),
    Unsubscribe(unsubscribe::Unsubscribe<'a>),
    Ping(ping::Ping<'a>),
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
impl_from_result!(nft_info, NFTInfo);
impl_from_result!(nft_buy_offers, NFTBuyOffers);
impl_from_result!(nft_sell_offers, NFTSellOffers);
impl_from_result!(nftoken, NFTokenMintResult);
impl_from_result!(no_ripple_check, NoRippleCheck);
impl_from_result!(path_find, PathFind);
impl_from_result!(random, Random);
impl_from_result!(ripple_path_find, RipplePathFind);
impl<'a> From<server_info::ServerInfo<'a>> for XRPLResult<'a> {
    fn from(value: server_info::ServerInfo<'a>) -> Self {
        XRPLResult::ServerInfo(Box::new(value))
    }
}
impl<'a> From<server_state::ServerState<'a>> for XRPLResult<'a> {
    fn from(value: server_state::ServerState<'a>) -> Self {
        XRPLResult::ServerState(Box::new(value))
    }
}
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
// AccountObjects: also accepts Other(Value) as a fallback because serde's
// ContentDeserializer (used by untagged enums) may fail to match AccountObjects
// when the xrpld response omits ledger_index (standalone/current-ledger mode).
// In that case the JSON lands in Other(Value) and we re-parse it here.
impl<'a> TryFrom<XRPLResult<'a>> for account_objects::AccountObjects<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::AccountObjects(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            res => Err(XRPLResultException::UnexpectedResultType(
                "AccountObjects".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
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
// LedgerData: serde's untagged enum may match LedgerClosed (which has fewer
// required fields) before reaching LedgerData. Re-serialize and re-parse
// to recover the full data.
impl<'a> TryFrom<XRPLResult<'a>> for ledger_data::LedgerData<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::LedgerData(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            other => {
                let value = serde_json::to_value(&other)?;
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }
}
// LedgerEntry: may match Ledger or LedgerClosed before reaching LedgerEntry.
impl<'a> TryFrom<XRPLResult<'a>> for ledger_entry::LedgerEntry<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::LedgerEntry(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            other => {
                let value = serde_json::to_value(&other)?;
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }
}
impl_try_from_result!(manifest, Manifest, Manifest);
// NFTBuyOffers and NFTSellOffers are structurally identical; the untagged enum
// always picks the first matching variant (NFTBuyOffers). Both TryFrom impls
// accept either variant so that nft_sell_offers and nft_buy_offers responses
// both parse correctly regardless of which variant serde selects.
impl<'a> TryFrom<XRPLResult<'a>> for nft_buy_offers::NFTBuyOffers<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTBuyOffers(value) => Ok(value),
            XRPLResult::NFTSellOffers(value) => Ok(nft_buy_offers::NFTBuyOffers {
                nft_id: value.nft_id,
                offers: value.offers,
                limit: value.limit,
                marker: value.marker,
            }),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTBuyOffers".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
impl<'a> TryFrom<XRPLResult<'a>> for nft_sell_offers::NFTSellOffers<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NFTSellOffers(value) => Ok(value),
            XRPLResult::NFTBuyOffers(value) => Ok(nft_sell_offers::NFTSellOffers {
                nft_id: value.nft_id,
                offers: value.offers,
                limit: value.limit,
                marker: value.marker,
            }),
            res => Err(XRPLResultException::UnexpectedResultType(
                "NFTSellOffers".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
impl_try_from_result!(nftoken, NFTokenMintResult, NFTokenMintResult);
// NoRippleCheck: may match Other(Value) due to untagged enum ordering.
impl<'a> TryFrom<XRPLResult<'a>> for no_ripple_check::NoRippleCheck<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::NoRippleCheck(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            other => {
                let value = serde_json::to_value(&other)?;
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }
}
impl_try_from_result!(path_find, PathFind, PathFind);
impl_try_from_result!(random, Random, Random);
// RipplePathFind: may match LedgerCurrent due to untagged enum ordering.
impl<'a> TryFrom<XRPLResult<'a>> for ripple_path_find::RipplePathFind<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::RipplePathFind(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            other => {
                let value = serde_json::to_value(&other)?;
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }
}
impl<'a> TryFrom<XRPLResult<'a>> for server_info::ServerInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::ServerInfo(value) => Ok(*value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            res => Err(XRPLResultException::UnexpectedResultType(
                "ServerInfo".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
impl<'a> TryFrom<XRPLResult<'a>> for server_state::ServerState<'a> {
    type Error = XRPLModelException;

    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::ServerState(value) => Ok(*value),
            res => Err(XRPLResultException::UnexpectedResultType(
                "ServerState".to_string(),
                res.get_name(),
            )
            .into()),
        }
    }
}
impl_try_from_result!(submit, Submit, Submit);
// SubmitMultisigned: may match Submit due to untagged enum ordering.
impl<'a> TryFrom<XRPLResult<'a>> for submit_multisigned::SubmitMultisigned<'a> {
    type Error = XRPLModelException;
    fn try_from(result: XRPLResult<'a>) -> XRPLModelResult<Self> {
        match result {
            XRPLResult::SubmitMultisigned(value) => Ok(value),
            XRPLResult::Other(XRPLOtherResult(ref value)) => {
                serde_json::from_value(value.clone()).map_err(Into::into)
            }
            other => {
                let value = serde_json::to_value(&other)?;
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }
}
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
            XRPLResult::NFTInfo(_) => "NFTInfo".to_string(),
            XRPLResult::NFTBuyOffers(_) => "NFTBuyOffers".to_string(),
            XRPLResult::NFTSellOffers(_) => "NFTSellOffers".to_string(),
            XRPLResult::NFTokenMintResult(_) => "NFTokenMintResult".to_string(),
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

/// Structured XRPL RPC error codes returned by xrpld/Clio.
///
/// Numeric values mirror xrpld's stable `ErrorCodeI` values. Use this enum
/// instead of matching response `error` strings such as `"txnNotFound"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
pub enum XRPLRpcError {
    Unknown = -1,
    BadSyntax = 1,
    JsonRpc = 2,
    Forbidden = 3,
    WrongNetwork = 4,
    NoPermission = 6,
    NoEvents = 7,
    TooBusy = 9,
    SlowDown = 10,
    HighFee = 11,
    NotEnabled = 12,
    NotReady = 13,
    AmendmentBlocked = 14,
    NoClosed = 15,
    NoCurrent = 16,
    NoNetwork = 17,
    NotSynced = 18,
    ActNotFound = 19,
    LgrNotFound = 21,
    LgrNotValidated = 22,
    MasterDisabled = 23,
    TxnNotFound = 29,
    InvalidHotwallet = 30,
    InvalidParams = 31,
    UnknownCommand = 32,
    NoPathRequest = 33,
    ActMalformed = 35,
    AlreadyMultisig = 36,
    AlreadySingleSig = 37,
    BadFeature = 40,
    BadIssuer = 41,
    BadMarket = 42,
    BadSecret = 43,
    BadSeed = 44,
    ChannelMalformed = 45,
    ChannelAmtMalformed = 46,
    CommandMissing = 47,
    DstActMalformed = 48,
    DstActMissing = 49,
    DstActNotFound = 50,
    DstAmtMalformed = 51,
    DstAmtMissing = 52,
    DstIsrMalformed = 53,
    LgrIdxsInvalid = 57,
    LgrIdxMalformed = 58,
    PublicMalformed = 62,
    SigningMalformed = 63,
    SendmaxMalformed = 64,
    SrcActMalformed = 65,
    SrcActMissing = 66,
    SrcActNotFound = 67,
    DelegateActNotFound = 68,
    SrcCurMalformed = 69,
    SrcIsrMalformed = 70,
    StreamMalformed = 71,
    AtxDeprecated = 72,
    Internal = 73,
    NotImpl = 74,
    NotSupported = 75,
    BadKeyType = 76,
    DbDeserialization = 77,
    ExcessiveLgrRange = 78,
    InvalidLgrRange = 79,
    ExpiredValidatorList = 80,
    ReportingUnsupported = 91,
    ObjectNotFound = 92,
    IssueMalformed = 93,
    OracleMalformed = 94,
    BadCredentials = 95,
    TxSigned = 96,
    DomainMalformed = 97,
    EntryNotFound = 98,
    UnexpectedLedgerType = 99,
}

impl TryFrom<i32> for XRPLRpcError {
    type Error = ();

    fn try_from(code: i32) -> Result<Self, Self::Error> {
        Ok(match code {
            -1 => XRPLRpcError::Unknown,
            1 => XRPLRpcError::BadSyntax,
            2 => XRPLRpcError::JsonRpc,
            3 => XRPLRpcError::Forbidden,
            4 => XRPLRpcError::WrongNetwork,
            6 => XRPLRpcError::NoPermission,
            7 => XRPLRpcError::NoEvents,
            9 => XRPLRpcError::TooBusy,
            10 => XRPLRpcError::SlowDown,
            11 => XRPLRpcError::HighFee,
            12 => XRPLRpcError::NotEnabled,
            13 => XRPLRpcError::NotReady,
            14 => XRPLRpcError::AmendmentBlocked,
            15 => XRPLRpcError::NoClosed,
            16 => XRPLRpcError::NoCurrent,
            17 => XRPLRpcError::NoNetwork,
            18 => XRPLRpcError::NotSynced,
            19 => XRPLRpcError::ActNotFound,
            21 => XRPLRpcError::LgrNotFound,
            22 => XRPLRpcError::LgrNotValidated,
            23 => XRPLRpcError::MasterDisabled,
            29 => XRPLRpcError::TxnNotFound,
            30 => XRPLRpcError::InvalidHotwallet,
            31 => XRPLRpcError::InvalidParams,
            32 => XRPLRpcError::UnknownCommand,
            33 => XRPLRpcError::NoPathRequest,
            35 => XRPLRpcError::ActMalformed,
            36 => XRPLRpcError::AlreadyMultisig,
            37 => XRPLRpcError::AlreadySingleSig,
            40 => XRPLRpcError::BadFeature,
            41 => XRPLRpcError::BadIssuer,
            42 => XRPLRpcError::BadMarket,
            43 => XRPLRpcError::BadSecret,
            44 => XRPLRpcError::BadSeed,
            45 => XRPLRpcError::ChannelMalformed,
            46 => XRPLRpcError::ChannelAmtMalformed,
            47 => XRPLRpcError::CommandMissing,
            48 => XRPLRpcError::DstActMalformed,
            49 => XRPLRpcError::DstActMissing,
            50 => XRPLRpcError::DstActNotFound,
            51 => XRPLRpcError::DstAmtMalformed,
            52 => XRPLRpcError::DstAmtMissing,
            53 => XRPLRpcError::DstIsrMalformed,
            57 => XRPLRpcError::LgrIdxsInvalid,
            58 => XRPLRpcError::LgrIdxMalformed,
            62 => XRPLRpcError::PublicMalformed,
            63 => XRPLRpcError::SigningMalformed,
            64 => XRPLRpcError::SendmaxMalformed,
            65 => XRPLRpcError::SrcActMalformed,
            66 => XRPLRpcError::SrcActMissing,
            67 => XRPLRpcError::SrcActNotFound,
            68 => XRPLRpcError::DelegateActNotFound,
            69 => XRPLRpcError::SrcCurMalformed,
            70 => XRPLRpcError::SrcIsrMalformed,
            71 => XRPLRpcError::StreamMalformed,
            72 => XRPLRpcError::AtxDeprecated,
            73 => XRPLRpcError::Internal,
            74 => XRPLRpcError::NotImpl,
            75 => XRPLRpcError::NotSupported,
            76 => XRPLRpcError::BadKeyType,
            77 => XRPLRpcError::DbDeserialization,
            78 => XRPLRpcError::ExcessiveLgrRange,
            79 => XRPLRpcError::InvalidLgrRange,
            80 => XRPLRpcError::ExpiredValidatorList,
            91 => XRPLRpcError::ReportingUnsupported,
            92 => XRPLRpcError::ObjectNotFound,
            93 => XRPLRpcError::IssueMalformed,
            94 => XRPLRpcError::OracleMalformed,
            95 => XRPLRpcError::BadCredentials,
            96 => XRPLRpcError::TxSigned,
            97 => XRPLRpcError::DomainMalformed,
            98 => XRPLRpcError::EntryNotFound,
            99 => XRPLRpcError::UnexpectedLedgerType,
            _ => return Err(()),
        })
    }
}

impl XRPLRpcError {
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "badSyntax" => XRPLRpcError::BadSyntax,
            "json_rpc" => XRPLRpcError::JsonRpc,
            "forbidden" => XRPLRpcError::Forbidden,
            "wrongNetwork" => XRPLRpcError::WrongNetwork,
            "noPermission" => XRPLRpcError::NoPermission,
            "noEvents" => XRPLRpcError::NoEvents,
            "tooBusy" => XRPLRpcError::TooBusy,
            "slowDown" => XRPLRpcError::SlowDown,
            "highFee" => XRPLRpcError::HighFee,
            "notEnabled" => XRPLRpcError::NotEnabled,
            "notReady" => XRPLRpcError::NotReady,
            "amendmentBlocked" => XRPLRpcError::AmendmentBlocked,
            "noClosed" => XRPLRpcError::NoClosed,
            "noCurrent" => XRPLRpcError::NoCurrent,
            "noNetwork" => XRPLRpcError::NoNetwork,
            "notSynced" => XRPLRpcError::NotSynced,
            "actNotFound" => XRPLRpcError::ActNotFound,
            "lgrNotFound" | "ledgerNotFound" => XRPLRpcError::LgrNotFound,
            "lgrNotValidated" => XRPLRpcError::LgrNotValidated,
            "masterDisabled" => XRPLRpcError::MasterDisabled,
            "txnNotFound" => XRPLRpcError::TxnNotFound,
            "invalidHotWallet" => XRPLRpcError::InvalidHotwallet,
            "invalidParams" => XRPLRpcError::InvalidParams,
            "unknownCmd" => XRPLRpcError::UnknownCommand,
            "noPathRequest" => XRPLRpcError::NoPathRequest,
            "actMalformed" => XRPLRpcError::ActMalformed,
            "alreadyMultisig" => XRPLRpcError::AlreadyMultisig,
            "alreadySingleSig" => XRPLRpcError::AlreadySingleSig,
            "badFeature" => XRPLRpcError::BadFeature,
            "badIssuer" => XRPLRpcError::BadIssuer,
            "badMarket" => XRPLRpcError::BadMarket,
            "badSecret" => XRPLRpcError::BadSecret,
            "badSeed" => XRPLRpcError::BadSeed,
            "channelMalformed" => XRPLRpcError::ChannelMalformed,
            "channelAmtMalformed" => XRPLRpcError::ChannelAmtMalformed,
            "commandMissing" => XRPLRpcError::CommandMissing,
            "dstActMalformed" => XRPLRpcError::DstActMalformed,
            "dstActMissing" => XRPLRpcError::DstActMissing,
            "dstActNotFound" => XRPLRpcError::DstActNotFound,
            "dstAmtMalformed" => XRPLRpcError::DstAmtMalformed,
            "dstAmtMissing" => XRPLRpcError::DstAmtMissing,
            "dstIsrMalformed" => XRPLRpcError::DstIsrMalformed,
            "lgrIdxsInvalid" => XRPLRpcError::LgrIdxsInvalid,
            "lgrIdxMalformed" => XRPLRpcError::LgrIdxMalformed,
            "publicMalformed" => XRPLRpcError::PublicMalformed,
            "signingMalformed" => XRPLRpcError::SigningMalformed,
            "sendMaxMalformed" => XRPLRpcError::SendmaxMalformed,
            "srcActMalformed" => XRPLRpcError::SrcActMalformed,
            "srcActMissing" => XRPLRpcError::SrcActMissing,
            "srcActNotFound" => XRPLRpcError::SrcActNotFound,
            "delegateActNotFound" => XRPLRpcError::DelegateActNotFound,
            "srcCurMalformed" => XRPLRpcError::SrcCurMalformed,
            "srcIsrMalformed" => XRPLRpcError::SrcIsrMalformed,
            "malformedStream" => XRPLRpcError::StreamMalformed,
            "deprecated" => XRPLRpcError::AtxDeprecated,
            "internal" => XRPLRpcError::Internal,
            "notImpl" => XRPLRpcError::NotImpl,
            "notSupported" => XRPLRpcError::NotSupported,
            "badKeyType" => XRPLRpcError::BadKeyType,
            "dbDeserialization" => XRPLRpcError::DbDeserialization,
            "excessiveLgrRange" => XRPLRpcError::ExcessiveLgrRange,
            "invalidLgrRange" => XRPLRpcError::InvalidLgrRange,
            "unlBlocked" => XRPLRpcError::ExpiredValidatorList,
            "objectNotFound" => XRPLRpcError::ObjectNotFound,
            "issueMalformed" => XRPLRpcError::IssueMalformed,
            "oracleMalformed" => XRPLRpcError::OracleMalformed,
            "badCredentials" => XRPLRpcError::BadCredentials,
            "transactionSigned" => XRPLRpcError::TxSigned,
            "domainMalformed" => XRPLRpcError::DomainMalformed,
            "entryNotFound" => XRPLRpcError::EntryNotFound,
            "unexpectedLedgerType" => XRPLRpcError::UnexpectedLedgerType,
            _ => return None,
        })
    }

    pub fn is_server_network_state(self) -> bool {
        matches!(
            self,
            XRPLRpcError::NoClosed
                | XRPLRpcError::NoCurrent
                | XRPLRpcError::NoNetwork
                | XRPLRpcError::NotSynced
                | XRPLRpcError::TooBusy
                | XRPLRpcError::SlowDown
        )
    }
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
    /// Raw JSON of the `result` field, preserved for fallback re-deserialization
    /// when the untagged `XRPLResult` enum matches the wrong variant.
    #[serde(skip)]
    pub raw_result: Option<Value>,
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
// AccountObjects: fallback to Other(Value) for the same reason as above.
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for account_objects::AccountObjects<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::AccountObjects(value) => Ok(value),
                XRPLResult::Other(XRPLOtherResult(ref value)) => {
                    serde_json::from_value(value.clone()).map_err(Into::into)
                }
                res => Err(XRPLResultException::UnexpectedResultType(
                    "AccountObjects".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
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
// LedgerData / LedgerEntry: use raw_result fallback for untagged enum
// mismatch where serde picks a wrong variant and loses fields.
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for ledger_data::LedgerData<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        if let Some(XRPLResult::LedgerData(value)) = response.result {
            return Ok(value);
        }
        // Fallback: re-deserialize from the raw result JSON
        match response.raw_result {
            Some(raw) => serde_json::from_value(raw).map_err(Into::into),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for ledger_entry::LedgerEntry<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        if let Some(XRPLResult::LedgerEntry(value)) = response.result {
            return Ok(value);
        }
        match response.raw_result {
            Some(raw) => serde_json::from_value(raw).map_err(Into::into),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl_try_from_response!(manifest, Manifest, Manifest);
impl_try_from_response!(nft_info, NFTInfo, NFTInfo);
// NFTBuyOffers and NFTSellOffers are structurally identical; the untagged enum
// always picks NFTBuyOffers first. Both TryFrom impls accept either variant.
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for nft_buy_offers::NFTBuyOffers<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::NFTBuyOffers(value) => Ok(value),
                XRPLResult::NFTSellOffers(value) => Ok(nft_buy_offers::NFTBuyOffers {
                    nft_id: value.nft_id,
                    offers: value.offers,
                    limit: value.limit,
                    marker: value.marker,
                }),
                res => Err(XRPLResultException::UnexpectedResultType(
                    "NFTBuyOffers".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for nft_sell_offers::NFTSellOffers<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::NFTSellOffers(value) => Ok(value),
                XRPLResult::NFTBuyOffers(value) => Ok(nft_sell_offers::NFTSellOffers {
                    nft_id: value.nft_id,
                    offers: value.offers,
                    limit: value.limit,
                    marker: value.marker,
                }),
                res => Err(XRPLResultException::UnexpectedResultType(
                    "NFTSellOffers".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl_try_from_response!(nftoken, NFTokenMintResult, NFTokenMintResult);
// NoRippleCheck / RipplePathFind: use raw_result fallback for untagged
// enum mismatch.
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for no_ripple_check::NoRippleCheck<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        if let Some(XRPLResult::NoRippleCheck(value)) = response.result {
            return Ok(value);
        }
        match response.raw_result {
            Some(raw) => serde_json::from_value(raw).map_err(Into::into),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl_try_from_response!(path_find, PathFind, PathFind);
impl_try_from_response!(ping, Ping, Ping);
impl_try_from_response!(random, Random, Random);
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for ripple_path_find::RipplePathFind<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        if let Some(XRPLResult::RipplePathFind(value)) = response.result {
            return Ok(value);
        }
        match response.raw_result {
            Some(raw) => serde_json::from_value(raw).map_err(Into::into),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl<'a> TryFrom<XRPLResponse<'a>> for server_info::ServerInfo<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::ServerInfo(value) => Ok(*value),
                XRPLResult::Other(XRPLOtherResult(ref value)) => {
                    serde_json::from_value(value.clone()).map_err(Into::into)
                }
                res => Err(XRPLResultException::UnexpectedResultType(
                    "ServerInfo".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl<'a> TryFrom<XRPLResponse<'a>> for server_state::ServerState<'a> {
    type Error = XRPLModelException;

    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        match response.result {
            Some(result) => match result {
                XRPLResult::ServerState(value) => Ok(*value),
                res => Err(XRPLResultException::UnexpectedResultType(
                    "ServerState".to_string(),
                    res.get_name(),
                )
                .into()),
            },
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
impl_try_from_response!(submit, Submit, Submit);
// SubmitMultisigned: may match Submit due to untagged enum ordering.
impl<'a, 'b> TryFrom<XRPLResponse<'a>> for submit_multisigned::SubmitMultisigned<'b>
where
    'a: 'b,
    'b: 'a,
{
    type Error = XRPLModelException;
    fn try_from(response: XRPLResponse<'a>) -> XRPLModelResult<Self> {
        if let Some(XRPLResult::SubmitMultisigned(value)) = response.result {
            return Ok(value);
        }
        match response.raw_result {
            Some(raw) => serde_json::from_value(raw).map_err(Into::into),
            None => Err(XRPLModelException::MissingField("result".to_string())),
        }
    }
}
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
                raw_result: None,
                status: None,
                r#type: None,
                warning: None,
                warnings: None,
            })
        } else {
            // Preserve the raw result JSON so that TryFrom impls can
            // re-deserialize when the untagged enum picks the wrong variant.
            let raw_result = map.remove("result");
            let result = raw_result
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok());
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
                result,
                raw_result,
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
    /// Return the structured RPC error code, if this response carries one.
    pub fn rpc_error(&self) -> Option<XRPLRpcError> {
        self.error_code
            .and_then(|code| XRPLRpcError::try_from(code).ok())
            .or_else(|| self.error.as_deref().and_then(XRPLRpcError::from_token))
    }

    pub fn is_success(&self) -> bool {
        if let Some(status) = &self.status {
            return status == &ResponseStatus::Success;
        }
        // Typed `XRPLResult` variants (e.g. `ServerInfo`, `Ping`) drop
        // unknown fields like `status` during deserialization. `raw_result`
        // preserves the original JSON, so prefer it; fall back to
        // re-serializing `result` only for responses constructed without a
        // raw payload (mostly tests).
        if let Some(raw) = &self.raw_result {
            if let Some(Value::String(status)) = raw.get("status") {
                return status == "success";
            }
        }
        if let Some(result) = &self.result {
            if let Ok(value) = serde_json::to_value(result) {
                if let Some(Value::String(status)) = value.get("status") {
                    return status == "success";
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPLWarning<'a> {
    pub id: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub forwarded: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fee_result() -> fee::Fee<'static> {
        fee::Fee {
            current_ledger_size: "14".into(),
            current_queue_size: "0".into(),
            drops: fee::Drops {
                base_fee: "10".into(),
                median_fee: "11000".into(),
                minimum_fee: "10".into(),
                open_ledger_fee: "10".into(),
            },
            expected_ledger_size: "24".into(),
            ledger_current_index: 26575101,
            levels: fee::Levels {
                median_level: "281600".into(),
                minimum_level: "256".into(),
                open_ledger_level: "256".into(),
                reference_level: "256".into(),
            },
            max_queue_size: None,
        }
    }

    fn random_result() -> random::Random<'static> {
        random::Random {
            random: "8ED765AEBBD6767603C2C9375B2679AEC76E6A8133EF59F04F9FC1AAA70E41AF".into(),
        }
    }

    #[test]
    fn test_from_into_xrpl_result() {
        let fee = fee_result();
        let result: XRPLResult = fee.clone().into();
        match &result {
            XRPLResult::Fee(f) => assert_eq!(f, &fee),
            _ => panic!("expected Fee variant"),
        }

        let random = random_result();
        let result: XRPLResult = random.clone().into();
        assert_eq!(result.get_name(), "Random");
    }

    #[test]
    fn test_try_from_xrpl_result_success() {
        let fee = fee_result();
        let result: XRPLResult = fee.clone().into();
        let recovered: fee::Fee = result.try_into().unwrap();
        assert_eq!(recovered, fee);
    }

    #[test]
    fn test_try_from_xrpl_result_wrong_variant() {
        let result: XRPLResult = random_result().into();
        let recovered: Result<fee::Fee, _> = result.try_into();
        assert!(recovered.is_err());
        let err = recovered.unwrap_err().to_string();
        assert!(err.contains("Fee"));
        assert!(err.contains("Random"));
    }

    #[test]
    fn test_get_name_for_variants() {
        let cases: &[(XRPLResult, &str)] = &[
            (XRPLResult::Fee(fee_result()), "Fee"),
            (XRPLResult::Random(random_result()), "Random"),
            (XRPLResult::Ping(ping::Ping::default()), "Ping"),
        ];
        for (result, expected) in cases {
            assert_eq!(result.get_name(), *expected);
        }

        // Other variant
        let other: XRPLResult = json!({"foo": "bar"}).into();
        assert_eq!(other.get_name(), "Other");
    }

    #[test]
    fn test_xrpl_other_result_get() {
        let other: XRPLOtherResult = json!({
            "value": 42,
            "name": "test"
        })
        .into();
        assert_eq!(other.get("value").and_then(|v| v.as_i64()), Some(42));
        assert!(other.get("missing").is_none());
        let v: u32 = other.try_get_typed("value").unwrap();
        assert_eq!(v, 42);
        let missing: Result<u32, _> = other.try_get_typed("missing");
        assert!(missing.is_err());
    }

    #[test]
    fn test_xrpl_other_result_try_from_xrpl_result() {
        let other: XRPLResult = json!({"x": 1}).into();
        let recovered: XRPLOtherResult = other.try_into().unwrap();
        assert_eq!(recovered.get("x").and_then(|v| v.as_i64()), Some(1));

        let fee: XRPLResult = fee_result().into();
        let recovered: Result<XRPLOtherResult, _> = fee.try_into();
        assert!(recovered.is_err());
    }

    #[test]
    fn test_response_deserialize_success() {
        let json = r#"{
            "result": {
                "current_ledger_size": "14",
                "current_queue_size": "0",
                "drops": {
                    "base_fee": "10",
                    "median_fee": "11000",
                    "minimum_fee": "10",
                    "open_ledger_fee": "10"
                },
                "expected_ledger_size": "24",
                "ledger_current_index": 26575101,
                "levels": {
                    "median_level": "281600",
                    "minimum_level": "256",
                    "open_ledger_level": "256",
                    "reference_level": "256"
                }
            },
            "status": "success",
            "type": "response"
        }"#;
        let response: XRPLResponse = serde_json::from_str(json).unwrap();
        assert!(response.is_success());
        assert_eq!(response.status, Some(ResponseStatus::Success));
        assert_eq!(response.r#type, Some(ResponseType::Response));
        assert!(response.result.is_some());
        assert!(response.raw_result.is_some());
        let fee: fee::Fee = response.try_into().unwrap();
        assert_eq!(fee.ledger_current_index, 26575101);
    }

    #[test]
    fn test_response_deserialize_error() {
        let json = r#"{
            "error": "noNetwork",
            "error_code": 17,
            "error_message": "Not synced to the network.",
            "status": "error"
        }"#;
        let response: XRPLResponse = serde_json::from_str(json).unwrap();
        assert!(!response.is_success());
        assert_eq!(response.error.as_deref(), Some("noNetwork"));
        assert_eq!(response.error_code, Some(17));
        assert_eq!(response.status, Some(ResponseStatus::Error));
        assert_eq!(response.rpc_error(), Some(XRPLRpcError::NoNetwork));

        // Try-into Result reports the error message.
        let result: Result<XRPLResult, _> = response.try_into();
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Not synced"));
    }

    #[test]
    fn test_response_deserialize_empty_fails() {
        let json = "{}";
        let response: Result<XRPLResponse, _> = serde_json::from_str(json);
        assert!(response.is_err());
    }

    #[test]
    fn test_response_deserialize_subscription_stream() {
        // Subscription stream items have no `result` and no `error_code`.
        let json = r#"{
            "type": "ledgerClosed",
            "ledger_index": 12345
        }"#;
        let response: XRPLResponse = serde_json::from_str(json).unwrap();
        // Stream items go into result as Other.
        assert!(response.result.is_some());
        // No status set for stream items
        assert!(response.status.is_none());
    }

    #[test]
    fn test_try_into_xrpl_result_no_result_field() {
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: None,
            raw_result: None,
            status: Some(ResponseStatus::Success),
            r#type: None,
            warning: None,
            warnings: None,
        };
        let result: Result<XRPLResult, _> = response.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_try_into_value() {
        let other: XRPLResult = json!({"k": "v"}).into();
        let value: Value = other.try_into().unwrap();
        assert_eq!(value.get("k").and_then(|v| v.as_str()), Some("v"));

        let fee_value: Value = XRPLResult::Fee(fee_result()).try_into().unwrap();
        assert_eq!(
            fee_value
                .get("ledger_current_index")
                .and_then(|v| v.as_u64()),
            Some(26575101)
        );
    }

    #[test]
    fn test_rpc_error_mapping_by_token_and_code() {
        let cases = [
            (-1, None, XRPLRpcError::Unknown),
            (1, Some("badSyntax"), XRPLRpcError::BadSyntax),
            (2, Some("json_rpc"), XRPLRpcError::JsonRpc),
            (3, Some("forbidden"), XRPLRpcError::Forbidden),
            (4, Some("wrongNetwork"), XRPLRpcError::WrongNetwork),
            (6, Some("noPermission"), XRPLRpcError::NoPermission),
            (7, Some("noEvents"), XRPLRpcError::NoEvents),
            (9, Some("tooBusy"), XRPLRpcError::TooBusy),
            (10, Some("slowDown"), XRPLRpcError::SlowDown),
            (11, Some("highFee"), XRPLRpcError::HighFee),
            (12, Some("notEnabled"), XRPLRpcError::NotEnabled),
            (13, Some("notReady"), XRPLRpcError::NotReady),
            (14, Some("amendmentBlocked"), XRPLRpcError::AmendmentBlocked),
            (15, Some("noClosed"), XRPLRpcError::NoClosed),
            (16, Some("noCurrent"), XRPLRpcError::NoCurrent),
            (17, Some("noNetwork"), XRPLRpcError::NoNetwork),
            (18, Some("notSynced"), XRPLRpcError::NotSynced),
            (19, Some("actNotFound"), XRPLRpcError::ActNotFound),
            (21, Some("lgrNotFound"), XRPLRpcError::LgrNotFound),
            (22, Some("lgrNotValidated"), XRPLRpcError::LgrNotValidated),
            (23, Some("masterDisabled"), XRPLRpcError::MasterDisabled),
            (29, Some("txnNotFound"), XRPLRpcError::TxnNotFound),
            (30, Some("invalidHotWallet"), XRPLRpcError::InvalidHotwallet),
            (31, Some("invalidParams"), XRPLRpcError::InvalidParams),
            (32, Some("unknownCmd"), XRPLRpcError::UnknownCommand),
            (33, Some("noPathRequest"), XRPLRpcError::NoPathRequest),
            (35, Some("actMalformed"), XRPLRpcError::ActMalformed),
            (36, Some("alreadyMultisig"), XRPLRpcError::AlreadyMultisig),
            (37, Some("alreadySingleSig"), XRPLRpcError::AlreadySingleSig),
            (40, Some("badFeature"), XRPLRpcError::BadFeature),
            (41, Some("badIssuer"), XRPLRpcError::BadIssuer),
            (42, Some("badMarket"), XRPLRpcError::BadMarket),
            (43, Some("badSecret"), XRPLRpcError::BadSecret),
            (44, Some("badSeed"), XRPLRpcError::BadSeed),
            (45, Some("channelMalformed"), XRPLRpcError::ChannelMalformed),
            (
                46,
                Some("channelAmtMalformed"),
                XRPLRpcError::ChannelAmtMalformed,
            ),
            (47, Some("commandMissing"), XRPLRpcError::CommandMissing),
            (48, Some("dstActMalformed"), XRPLRpcError::DstActMalformed),
            (49, Some("dstActMissing"), XRPLRpcError::DstActMissing),
            (50, Some("dstActNotFound"), XRPLRpcError::DstActNotFound),
            (51, Some("dstAmtMalformed"), XRPLRpcError::DstAmtMalformed),
            (52, Some("dstAmtMissing"), XRPLRpcError::DstAmtMissing),
            (53, Some("dstIsrMalformed"), XRPLRpcError::DstIsrMalformed),
            (57, Some("lgrIdxsInvalid"), XRPLRpcError::LgrIdxsInvalid),
            (58, Some("lgrIdxMalformed"), XRPLRpcError::LgrIdxMalformed),
            (62, Some("publicMalformed"), XRPLRpcError::PublicMalformed),
            (63, Some("signingMalformed"), XRPLRpcError::SigningMalformed),
            (64, Some("sendMaxMalformed"), XRPLRpcError::SendmaxMalformed),
            (65, Some("srcActMalformed"), XRPLRpcError::SrcActMalformed),
            (66, Some("srcActMissing"), XRPLRpcError::SrcActMissing),
            (67, Some("srcActNotFound"), XRPLRpcError::SrcActNotFound),
            (
                68,
                Some("delegateActNotFound"),
                XRPLRpcError::DelegateActNotFound,
            ),
            (69, Some("srcCurMalformed"), XRPLRpcError::SrcCurMalformed),
            (70, Some("srcIsrMalformed"), XRPLRpcError::SrcIsrMalformed),
            (71, Some("malformedStream"), XRPLRpcError::StreamMalformed),
            (72, Some("deprecated"), XRPLRpcError::AtxDeprecated),
            (73, Some("internal"), XRPLRpcError::Internal),
            (74, Some("notImpl"), XRPLRpcError::NotImpl),
            (75, Some("notSupported"), XRPLRpcError::NotSupported),
            (76, Some("badKeyType"), XRPLRpcError::BadKeyType),
            (
                77,
                Some("dbDeserialization"),
                XRPLRpcError::DbDeserialization,
            ),
            (
                78,
                Some("excessiveLgrRange"),
                XRPLRpcError::ExcessiveLgrRange,
            ),
            (79, Some("invalidLgrRange"), XRPLRpcError::InvalidLgrRange),
            (80, Some("unlBlocked"), XRPLRpcError::ExpiredValidatorList),
            (91, None, XRPLRpcError::ReportingUnsupported),
            (92, Some("objectNotFound"), XRPLRpcError::ObjectNotFound),
            (93, Some("issueMalformed"), XRPLRpcError::IssueMalformed),
            (94, Some("oracleMalformed"), XRPLRpcError::OracleMalformed),
            (95, Some("badCredentials"), XRPLRpcError::BadCredentials),
            (96, Some("transactionSigned"), XRPLRpcError::TxSigned),
            (97, Some("domainMalformed"), XRPLRpcError::DomainMalformed),
            (98, Some("entryNotFound"), XRPLRpcError::EntryNotFound),
            (
                99,
                Some("unexpectedLedgerType"),
                XRPLRpcError::UnexpectedLedgerType,
            ),
        ];

        for (code, token, error) in cases {
            assert_eq!(XRPLRpcError::try_from(code), Ok(error));
            if let Some(token) = token {
                assert_eq!(XRPLRpcError::from_token(token), Some(error));
            }
        }
        assert_eq!(
            XRPLRpcError::from_token("ledgerNotFound"),
            Some(XRPLRpcError::LgrNotFound)
        );
        assert!(XRPLRpcError::try_from(5).is_err());
        assert_eq!(XRPLRpcError::from_token("notARealToken"), None);
    }

    #[test]
    fn test_rpc_error_server_network_state() {
        assert!(XRPLRpcError::NoClosed.is_server_network_state());
        assert!(XRPLRpcError::NoCurrent.is_server_network_state());
        assert!(XRPLRpcError::NoNetwork.is_server_network_state());
        assert!(XRPLRpcError::NotSynced.is_server_network_state());
        assert!(XRPLRpcError::TooBusy.is_server_network_state());
        assert!(XRPLRpcError::SlowDown.is_server_network_state());
        assert!(!XRPLRpcError::TxnNotFound.is_server_network_state());
        assert!(!XRPLRpcError::InvalidParams.is_server_network_state());
    }

    #[test]
    fn test_response_status_serde() {
        assert_eq!(
            serde_json::to_string(&ResponseStatus::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::from_str::<ResponseStatus>("\"error\"").unwrap(),
            ResponseStatus::Error
        );
    }

    #[test]
    fn test_response_type_serde() {
        // Uses camelCase
        assert_eq!(
            serde_json::to_string(&ResponseType::LedgerClosed).unwrap(),
            "\"ledgerClosed\""
        );
        assert_eq!(
            serde_json::from_str::<ResponseType>("\"transaction\"").unwrap(),
            ResponseType::Transaction
        );
    }

    #[test]
    fn test_is_success_inferred_from_result_status_field() {
        // No top-level status. The result is `Other(Value)` — its serialized
        // form preserves the inner `status: success` field, so `is_success`
        // can find it.
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: Some(json!({"status": "success", "foo": "bar"}).into()),
            raw_result: None,
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(response.is_success());

        // status: "error" inside the result → not success
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: Some(json!({"status": "error"}).into()),
            raw_result: None,
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(!response.is_success());
    }

    #[test]
    fn test_response_with_no_status_or_result_is_not_success() {
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: None,
            raw_result: None,
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(!response.is_success());
    }

    #[test]
    fn test_is_success_uses_raw_result_branch() {
        // raw_result with `status: "success"` returns true even when `status`
        // and `result` are absent — this is the path typed XRPLResult
        // variants (e.g. ServerInfo) rely on, since they drop unknown fields
        // like `status` during deserialization.
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: None,
            raw_result: Some(json!({"status": "success", "info": "x"})),
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(response.is_success());

        // raw_result without a `status` field falls through to false.
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: None,
            raw_result: Some(json!({"info": "no status"})),
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(!response.is_success());
    }

    #[test]
    fn test_is_success_falls_through_when_typed_result_lacks_status() {
        // A typed result variant serializes without a `status` field, so the
        // inner pattern in the `result` fallback branch never matches and
        // control falls through to `false`.
        let response = XRPLResponse {
            id: None,
            error: None,
            error_code: None,
            error_message: None,
            forwarded: None,
            request: None,
            result: Some(XRPLResult::Fee(fee_result())),
            raw_result: None,
            status: None,
            r#type: None,
            warning: None,
            warnings: None,
        };
        assert!(!response.is_success());
    }
}
