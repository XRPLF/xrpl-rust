use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The ping command returns an acknowledgement, so that
/// clients can test the connection status and latency.
///
/// See Ping:
/// `<https://xrpl.org/ping.html#ping>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ping<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
}

impl<'a> Model for Ping<'a> {}

impl<'a> Request<'a> for Ping<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Ping<'a> {
    pub fn new(id: Option<Cow<'a, str>>) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Ping,
                id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_serde_round_trip() {
        let req = Ping::new(Some("ping-1".into()));
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: Ping = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"ping\""));
    }

    #[test]
    fn test_get_common_fields() {
        let mut req = Ping::new(None);
        assert!(req.get_common_fields().id.is_none());
        req.get_common_fields_mut().id = Some("x".to_string().into());
        assert_eq!(req.get_common_fields().id.as_deref(), Some("x"));
    }
}
