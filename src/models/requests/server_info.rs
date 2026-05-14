use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The server_info command asks the server for a
/// human-readable version of various information about the
/// rippled server being queried.
///
/// See Server Info:
/// `<https://xrpl.org/server_info.html#server_info>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ServerInfo<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
}

impl<'a> Model for ServerInfo<'a> {}

impl<'a> Request<'a> for ServerInfo<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> ServerInfo<'a> {
    pub fn new(id: Option<Cow<'a, str>>) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::ServerInfo,
                id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let req = ServerInfo::new(Some("si-1".into()));
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: ServerInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"server_info\""));
    }
}
