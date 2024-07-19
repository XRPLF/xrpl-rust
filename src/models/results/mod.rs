use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};
use serde::{Deserialize, Serialize};

mod fee;
pub use fee::{Fee, *};
use serde_json::{Map, Value};

use super::requests::XRPLRequest;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum XRPLResult<'a> {
    Fee(Fee<'a>),
    Custom(Value),
}

impl<'a> From<Fee<'a>> for XRPLResult<'a> {
    fn from(fee: Fee<'a>) -> Self {
        XRPLResult::Fee(fee)
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

impl<'a> XRPLResponse<'a> {
    pub fn is_success(&self) -> bool {
        self.status == Some(ResponseStatus::Success)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPLWarning<'a> {
    pub id: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub forwarded: Option<bool>,
}
