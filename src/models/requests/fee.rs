use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The fee command reports the current state of the open-ledger
/// requirements for the transaction cost. This requires the
/// FeeEscalation amendment to be enabled. This is a public
/// command available to unprivileged users.
///
/// See Fee:
/// `<https://xrpl.org/fee.html#fee>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Fee<'a> {
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
    /// The request method.
    #[serde(default = "RequestMethod::fee")]
    pub command: RequestMethod,
}

impl<'a> Default for Fee<'a> {
    fn default() -> Self {
        Fee {
            id: None,
            command: RequestMethod::Fee,
        }
    }
}

impl<'a> Model for Fee<'a> {}

impl<'a> Fee<'a> {
    pub fn new(id: Option<Cow<'a, str>>) -> Self {
        Self {
            id,
            command: RequestMethod::Fee,
        }
    }
}
