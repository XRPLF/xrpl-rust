use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The server_state command asks the server for various
/// machine-readable information about the rippled server's
/// current state. The response is almost the same as the
/// server_info method, but uses units that are easier to
/// process instead of easier to read. (For example, XRP
/// values are given in integer drops instead of scientific
/// notation or decimal values, and time is given in
/// milliseconds instead of seconds.)
///
/// See Server State:
/// `<https://xrpl.org/server_state.html#server_state>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ServerState<'a> {
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::server_state")]
    pub command: RequestMethod,
}

impl<'a> Default for ServerState<'a> {
    fn default() -> Self {
        ServerState {
            id: None,
            command: RequestMethod::ServerState,
        }
    }
}

impl<'a> Model for ServerState<'a> {}

impl<'a> ServerState<'a> {
    fn new(id: Option<&'a str>) -> Self {
        Self {
            id,
            command: RequestMethod::ServerState,
        }
    }
}
