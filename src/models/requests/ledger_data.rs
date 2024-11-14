use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

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
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// Limit the number of ledger objects to retrieve.
    /// The server is not required to honor this value.
    pub limit: Option<u16>,
    /// Value from a previous paginated response.
    /// Resume retrieving data where that response left off.
    pub marker: Option<u32>,
}

impl<'a> Model for LedgerData<'a> {}

impl<'a> Request<'a> for LedgerData<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> LedgerData<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        binary: Option<bool>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        limit: Option<u16>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::LedgerData,
                id,
            },
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            binary,
            limit,
            marker,
        }
    }
}
