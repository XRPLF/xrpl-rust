use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// The ledger_closed method returns the unique identifiers of
/// the most recently closed ledger. (This ledger is not
/// necessarily validated and immutable yet.)
///
/// See Ledger Closed:
/// `<https://xrpl.org/ledger_closed.html#ledger_closed>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LedgerClosed<'a> {
    /// The unique request id.
    pub id: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::ledger_closed")]
    pub command: RequestMethod,
}

impl<'a> Default for LedgerClosed<'a> {
    fn default() -> Self {
        LedgerClosed {
            id: None,
            command: RequestMethod::LedgerClosed,
        }
    }
}

impl<'a> Model for LedgerClosed<'a> {}

impl<'a> LedgerClosed<'a> {
    fn new(id: Option<&'a str>) -> Self {
        Self {
            id,
            command: RequestMethod::LedgerClosed,
        }
    }
}
