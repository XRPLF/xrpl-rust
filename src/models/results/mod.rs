pub mod account_info;
pub mod account_tx;
pub mod exceptions;
pub mod fee;
pub mod ledger;
pub mod server_state;
pub mod submit;
pub mod tx;

use super::requests::XRPLRequest;
use crate::Err;
use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec::Vec,
};
use anyhow::Result;
use core::convert::{TryFrom, TryInto};
use exceptions::XRPLResultException;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{value::Index, Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum XRPLOptionalResult<T> {
    Result(T),
    Other(XRPLOtherResult),
}

impl<T> XRPLOptionalResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            XRPLOptionalResult::Result(result) => result,
            XRPLOptionalResult::Other(_) => {
                panic!("{}", XRPLResultException::UnwrapOnOther.to_string())
            }
        }
    }

    /// Try to convert the result into an expected XRPL result.
    pub fn try_into_result(self) -> Result<T> {
        match self {
            XRPLOptionalResult::Result(result) => Ok(result),
            XRPLOptionalResult::Other(other) => Err!(XRPLResultException::ExpectedResult(other)),
        }
    }

    /// Get a value from the result by index.
    pub fn try_get_typed<I, U>(&self, index: I) -> Result<U>
    where
        T: Serialize,
        I: Index,
        U: DeserializeOwned,
    {
        match self {
            XRPLOptionalResult::Result(result) => match serde_json::to_value(result) {
                Ok(value) => match value.get(index) {
                    Some(value) => match serde_json::from_value(value.to_owned()) {
                        Ok(value) => Ok(value),
                        Err(e) => Err!(e),
                    },
                    None => Err!(XRPLResultException::IndexNotFound),
                },
                Err(e) => Err!(e),
            },
            XRPLOptionalResult::Other(other) => other.try_get_typed(index),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XRPLOtherResult(Value);

impl TryFrom<XRPLResult<'_>> for XRPLOtherResult {
    type Error = anyhow::Error;

    fn try_from(result: XRPLResult) -> Result<Self> {
        match result {
            XRPLResult::Other(value) => Ok(value),
            res => Err!(XRPLResultException::UnexpectedResultType(
                "Other".to_string(),
                res.get_name()
            )),
        }
    }
}

impl From<Value> for XRPLOtherResult {
    fn from(value: Value) -> Self {
        XRPLOtherResult(value)
    }
}

impl Into<Value> for XRPLOtherResult {
    fn into(self) -> Value {
        self.0
    }
}

impl XRPLOtherResult {
    pub fn get(&self, index: impl Index) -> Option<&Value> {
        self.0.get(index)
    }

    pub fn try_get_typed<I, T>(&self, index: I) -> Result<T>
    where
        I: Index,
        T: DeserializeOwned,
    {
        match self.0.get(index) {
            Some(value) => match serde_json::from_value(value.clone()) {
                Ok(value) => Ok(value),
                Err(e) => Err!(e),
            },
            None => Err!(XRPLResultException::IndexNotFound),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum XRPLResult<'a> {
    AccountInfo(account_info::AccountInfo<'a>),
    AccountTx(account_tx::AccountTx<'a>),
    Fee(fee::Fee<'a>),
    Ledger(ledger::Ledger<'a>),
    ServerState(server_state::ServerState<'a>),
    Submit(submit::Submit<'a>),
    Tx(tx::Tx<'a>),
    Other(XRPLOtherResult),
}

impl<'a> From<account_info::AccountInfo<'a>> for XRPLResult<'a> {
    fn from(account_info: account_info::AccountInfo<'a>) -> Self {
        XRPLResult::AccountInfo(account_info)
    }
}

impl<'a> From<account_tx::AccountTx<'a>> for XRPLResult<'a> {
    fn from(account_tx: account_tx::AccountTx<'a>) -> Self {
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
        XRPLResult::ServerState(server_state)
    }
}

impl<'a> From<submit::Submit<'a>> for XRPLResult<'a> {
    fn from(submit: submit::Submit<'a>) -> Self {
        XRPLResult::Submit(submit)
    }
}

impl<'a> From<tx::Tx<'a>> for XRPLResult<'a> {
    fn from(tx: tx::Tx<'a>) -> Self {
        XRPLResult::Tx(tx)
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
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Value> {
        match self {
            XRPLResult::Other(XRPLOtherResult(value)) => Ok(value),
            res => match serde_json::to_value(res) {
                Ok(value) => Ok(value),
                Err(e) => Err!(e),
            },
        }
    }
}

impl XRPLResult<'_> {
    pub(crate) fn get_name(&self) -> String {
        match self {
            XRPLResult::AccountInfo(_) => "AccountInfo".to_string(),
            XRPLResult::AccountTx(_) => "AccountTx".to_string(),
            XRPLResult::Fee(_) => "Fee".to_string(),
            XRPLResult::Ledger(_) => "ledger-models".to_string(),
            XRPLResult::ServerState(_) => "ServerState".to_string(),
            XRPLResult::Submit(_) => "Submit".to_string(),
            XRPLResult::Tx(_) => "Tx".to_string(),
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
    fn deserialize<D>(deserializer: D) -> Result<XRPLResponse<'a>, D::Error>
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
                id: map.remove("id").map(|item| match item.as_str() {
                    Some(item_str) => Cow::Owned(item_str.to_string()),
                    None => Cow::Borrowed(""),
                }),
                error: map.remove("error").map(|item| match item.as_str() {
                    Some(item_str) => Cow::Owned(item_str.to_string()),
                    None => Cow::Borrowed(""),
                }),
                error_code: map
                    .remove("error_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32),
                error_message: map.remove("error_message").map(|item| match item.as_str() {
                    Some(item_str) => Cow::Owned(item_str.to_string()),
                    None => Cow::Borrowed(""),
                }),
                forwarded: map.remove("forwarded").and_then(|v| v.as_bool()),
                request: map
                    .remove("request")
                    .map(|v| serde_json::from_value(v).unwrap()),
                result: map
                    .remove("result")
                    .map(|v| serde_json::from_value(v).unwrap()),
                status: map
                    .remove("status")
                    .map(|v| serde_json::from_value(v).unwrap()),
                r#type: map
                    .remove("type")
                    .map(|v| serde_json::from_value(v).unwrap()),
                warning: map.remove("warning").map(|item| match item.as_str() {
                    Some(item_str) => Cow::Owned(item_str.to_string()),
                    None => Cow::Borrowed(""),
                }),
                warnings: map
                    .remove("warnings")
                    .and_then(|v| serde_json::from_value(v).ok()),
            })
        }
    }
}

impl TryInto<Value> for XRPLResponse<'_> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Value> {
        match serde_json::to_value(self) {
            Ok(value) => Ok(value),
            Err(e) => Err!(e),
        }
    }
}

impl<'a> XRPLResponse<'a> {
    pub fn is_success(&self) -> bool {
        if let Some(status) = &self.status {
            status == &ResponseStatus::Success
        } else {
            if let Some(result) = &self.result {
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

    pub fn try_into_opt_result<T>(self) -> Result<XRPLOptionalResult<T>>
    where
        T: TryFrom<XRPLResult<'a>, Error = anyhow::Error>,
    {
        match self.result {
            Some(result) => match result.clone().try_into() {
                Ok(result) => Ok(XRPLOptionalResult::Result(result)),
                Err(_) => Ok(XRPLOptionalResult::Other(result.try_into()?)),
            },
            None => {
                if let Some(error) = self.error {
                    Err!(XRPLResultException::ResponseError(format!(
                        "{}: {}",
                        error,
                        self.error_message.unwrap_or_default()
                    )))
                } else {
                    Err!(XRPLResultException::ExpectedResultOrError)
                }
            }
        }
    }

    pub fn try_into_result<T>(self) -> Result<T>
    where
        T: TryFrom<XRPLResult<'a>, Error = anyhow::Error>,
    {
        match self.try_into_opt_result()? {
            XRPLOptionalResult::Result(result) => Ok(result),
            XRPLOptionalResult::Other(other) => Err!(XRPLResultException::ExpectedResult(other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPLWarning<'a> {
    pub id: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub forwarded: Option<bool>,
}
