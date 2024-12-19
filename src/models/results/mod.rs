pub mod account_info;
pub mod account_tx;
pub mod exceptions;
pub mod fee;
pub mod ledger;
pub mod nftoken;
pub mod server_state;
pub mod submit;
pub mod tx;

use super::{requests::XRPLRequest, XRPLModelException, XRPLModelResult};
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
    AccountInfo(account_info::AccountInfoMap<'a>),
    AccountTx(account_tx::AccountTxMap<'a>),
    Fee(fee::Fee<'a>),
    Ledger(ledger::Ledger<'a>),
    ServerState(Box<server_state::ServerState<'a>>), // Boxed because ServerState is large
    Submit(submit::Submit<'a>),
    Tx(tx::Tx<'a>),
    NFTokenMint(nftoken::NFTokenMintResult),
    Other(XRPLOtherResult),
}

impl<'a> From<account_info::AccountInfo<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfo<'a>) -> Self {
        XRPLResult::AccountInfo(account_info::AccountInfoMap::Default(account_info))
    }
}

impl<'a> From<account_info::AccountInfoV1<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfoV1<'a>) -> Self {
        XRPLResult::AccountInfo(account_info::AccountInfoMap::V1(account_info))
    }
}

impl<'a> From<account_info::AccountInfoMap<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfoMap<'a>) -> Self {
        XRPLResult::AccountInfo(account_info)
    }
}

impl<'a> From<account_tx::AccountTx<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTx<'a>) -> Self {
        XRPLResult::AccountTx(account_tx::AccountTxMap::Default(account_tx))
    }
}

impl<'a> From<account_tx::AccountTxV1<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTxV1<'a>) -> Self {
        XRPLResult::AccountTx(account_tx::AccountTxMap::V1(account_tx))
    }
}

impl<'a> From<account_tx::AccountTxMap<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTxMap<'a>) -> Self {
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
        XRPLResult::Tx(tx::TxMap::Default(tx))
    }
}

impl<'a> From<tx::TxV1<'a>> for XRPLResult<'a> {
    fn from(tx: tx::TxV1<'a>) -> Self {
        XRPLResult::Tx(tx::TxMap::V1(tx))
    }
}

impl<'a> From<tx::TxMap<'a>> for XRPLResult<'a> {
    fn from(tx: tx::TxMap<'a>) -> Self {
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

impl TryInto<Value> for XRPLResponse<'_> {
    type Error = XRPLModelException;

    fn try_into(self) -> XRPLModelResult<Value> {
        Ok(serde_json::to_value(self)?)
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
