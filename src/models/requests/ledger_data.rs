use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The ledger_data method retrieves contents of the specified
/// ledger. You can iterate through several calls to retrieve
/// the entire contents of a single ledger version.
///
/// See Ledger Data:
/// `<https://xrpl.org/ledger_data.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LedgerData<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// If set to true, return ledger objects as hashed hex
    /// strings instead of JSON.
    pub binary: Option<bool>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// Limit the number of ledger objects to retrieve.
    /// The server is not required to honor this value.
    pub limit: Option<u16>,
    /// Value from a previous paginated response.
    /// Resume retrieving data where that response left off.
    pub marker: Option<u32>,
}

impl<'a> Model for LedgerData<'a> {}

impl<'a> Request for LedgerData<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> LedgerData<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        binary: Option<bool>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        limit: Option<u16>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::LedgerData,
                id,
            },
            ledger_hash,
            ledger_index,
            binary,
            limit,
            marker,
        }
    }
}
