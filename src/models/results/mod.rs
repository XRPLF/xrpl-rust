pub mod account_info;
pub mod account_tx;
pub mod exceptions;
pub mod fee;
pub mod ledger;
pub mod nft_buy_offer;
pub mod nft_history;
pub mod nft_info;
pub mod nft_sell_offers;
pub mod nftoken;
pub mod nfts_by_issuer;
pub mod server_state;
pub mod submit;
pub mod tx;

use super::{requests::XRPLRequest, Amount, XRPLModelException, XRPLModelResult};
use alloc::{
    borrow::Cow,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
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
    AccountInfo(account_info::AccountInfoVersionMap<'a>),
    AccountTx(account_tx::AccountTxVersionMap<'a>),
    Fee(fee::Fee<'a>),
    Ledger(ledger::Ledger<'a>),
    NFTBuyOffer(nft_buy_offer::NFTBuyOffer<'a>),
    NFTHistory(nft_history::NFTHistory<'a>),
    NFTInfo(nft_info::NFTInfo<'a>),
    NFTSellOffers(nft_sell_offers::NFTSellOffer<'a>),
    NFTsByIssuer(nfts_by_issuer::NFTsByIssuer<'a>),
    ServerState(Box<server_state::ServerState<'a>>), // Boxed because ServerState is large
    Submit(submit::Submit<'a>),
    Tx(tx::Tx<'a>),
    NFTokenMint(nftoken::NFTokenMintResult),
    Other(XRPLOtherResult),
}

impl<'a> From<account_info::AccountInfo<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfo<'a>) -> Self {
        XRPLResult::AccountInfo(account_info::AccountInfoVersionMap::Default(account_info))
    }
}

impl<'a> From<account_info::AccountInfoV1<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfoV1<'a>) -> Self {
        XRPLResult::AccountInfo(account_info::AccountInfoVersionMap::V1(account_info))
    }
}

impl<'a> From<account_info::AccountInfoVersionMap<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfoVersionMap<'a>) -> Self {
        XRPLResult::AccountInfo(account_info)
    }
}

impl<'a> From<account_tx::AccountTx<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTx<'a>) -> Self {
        XRPLResult::AccountTx(account_tx::AccountTxVersionMap::Default(account_tx))
    }
}

impl<'a> From<account_tx::AccountTxV1<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTxV1<'a>) -> Self {
        XRPLResult::AccountTx(account_tx::AccountTxVersionMap::V1(account_tx))
    }
}

impl<'a> From<account_tx::AccountTxVersionMap<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTxVersionMap<'a>) -> Self {
        XRPLResult::AccountTx(account_tx)
    }
}

impl<'a> From<fee::Fee<'a>> for XRPLResult<'a> {
    fn from(fee: fee::Fee<'a>) -> Self {
        XRPLResult::Fee(fee)
    }
}

impl<'a> From<ledger::Ledger<'a>> for XRPLResult<'a> {
    fn from(ledger: ledger::Ledger<'a>) -> Self {
        XRPLResult::Ledger(ledger)
    }
}

impl<'a> From<server_state::ServerState<'a>> for XRPLResult<'a> {
    fn from(server_state: server_state::ServerState<'a>) -> Self {
        XRPLResult::ServerState(Box::new(server_state))
    }
}

impl<'a> From<submit::Submit<'a>> for XRPLResult<'a> {
    fn from(submit: submit::Submit<'a>) -> Self {
        XRPLResult::Submit(submit)
    }
}

impl<'a> From<tx::Tx<'a>> for XRPLResult<'a> {
    fn from(tx: tx::Tx<'a>) -> Self {
        XRPLResult::Tx(tx::TxVersionMap::Default(tx))
    }
}

impl<'a> From<tx::TxV1<'a>> for XRPLResult<'a> {
    fn from(tx: tx::TxV1<'a>) -> Self {
        XRPLResult::Tx(tx::TxVersionMap::V1(tx))
    }
}

impl<'a> From<tx::TxVersionMap<'a>> for XRPLResult<'a> {
    fn from(tx: tx::TxVersionMap<'a>) -> Self {
        XRPLResult::Tx(tx)
    }
}

impl<'a> From<nftoken::NFTokenMintResult> for XRPLResult<'a> {
    fn from(result: nftoken::NFTokenMintResult) -> Self {
        XRPLResult::NFTokenMint(result)
    }
}

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

impl<'a> From<nft_buy_offer::NFTBuyOffer<'a>> for XRPLResult<'a> {
    fn from(nft_buy_offer: nft_buy_offer::NFTBuyOffer<'a>) -> Self {
        XRPLResult::NFTBuyOffer(nft_buy_offer)
    }
}

impl<'a> From<nft_history::NFTHistory<'a>> for XRPLResult<'a> {
    fn from(nft_history: nft_history::NFTHistory<'a>) -> Self {
        XRPLResult::NFTHistory(nft_history)
    }
}

impl<'a> From<nft_info::NFTInfo<'a>> for XRPLResult<'a> {
    fn from(nft_info: nft_info::NFTInfo<'a>) -> Self {
        XRPLResult::NFTInfo(nft_info)
    }
}

impl<'a> From<nft_sell_offers::NFTSellOffer<'a>> for XRPLResult<'a> {
    fn from(nft_sell_offers: nft_sell_offers::NFTSellOffer<'a>) -> Self {
        XRPLResult::NFTSellOffers(nft_sell_offers)
    }
}

impl<'a> From<nfts_by_issuer::NFTsByIssuer<'a>> for XRPLResult<'a> {
    fn from(nfts_by_issuer: nfts_by_issuer::NFTsByIssuer<'a>) -> Self {
        XRPLResult::NFTsByIssuer(nfts_by_issuer)
    }
}

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
            XRPLResult::AccountInfo(_) => "AccountInfo".to_string(),
            XRPLResult::AccountTx(_) => "AccountTx".to_string(),
            XRPLResult::Fee(_) => "Fee".to_string(),
            XRPLResult::Ledger(_) => "Ledger".to_string(),
            XRPLResult::NFTBuyOffer(_) => "NFTBuyOffer".to_string(),
            XRPLResult::NFTHistory(_) => "NFTHistory".to_string(),
            XRPLResult::NFTInfo(_) => "NFTInfo".to_string(),
            XRPLResult::NFTSellOffers(_) => "NFTSellOffers".to_string(),
            XRPLResult::NFTsByIssuer(_) => "NFTsByIssuer".to_string(),
            XRPLResult::ServerState(_) => "ServerState".to_string(),
            XRPLResult::Submit(_) => "Submit".to_string(),
            XRPLResult::Tx(_) => "Tx".to_string(),
            XRPLResult::NFTokenMint(_) => "NFTokenMint".to_string(),
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
    pub warnings: Option<Vec<XRPLWarning<'a>>>,
}

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
