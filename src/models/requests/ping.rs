use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The ping command returns an acknowledgement, so that
/// clients can test the connection status and latency.
///
/// See Ping:
/// `<https://xrpl.org/ping.html#ping>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ping<'a> {
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::ping")]
    pub command: RequestMethod,
}

impl<'a> Default for Ping<'a> {
    fn default() -> Self {
        Ping {
            id: None,
            command: RequestMethod::Ping,
        }
    }
}

impl<'a> Model for Ping<'a> {}

impl<'a> Ping<'a> {
    pub fn new(id: Option<&'a str>) -> Self {
        Self {
            id,
            command: RequestMethod::Ping,
        }
    }
}
