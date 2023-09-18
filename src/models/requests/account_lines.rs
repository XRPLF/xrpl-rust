use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This request returns information about an account's trust
/// lines, including balances in all non-XRP currencies and
/// assets. All information retrieved is relative to a particular
/// version of the ledger.
///
/// See Account Lines:
/// `<https://xrpl.org/account_lines.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountLines<'a> {
    /// A unique identifier for the account, most commonly the
    /// account's Address.
    pub account: Cow<'a, str>,
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// Limit the number of trust lines to retrieve. The server
    /// is not required to honor this value. Must be within the
    /// inclusive range 10 to 400.
    pub limit: Option<u16>,
    /// The Address of a second account. If provided, show only
    /// lines of trust connecting the two accounts.
    pub peer: Option<Cow<'a, str>>,
    /// Value from a previous paginated response. Resume retrieving
    /// data where that response left off.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::account_lines")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountLines<'a> {
    fn default() -> Self {
        AccountLines {
            account: "".into(),
            id: None,
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            peer: None,
            marker: None,
            command: RequestMethod::AccountLines,
        }
    }
}

impl<'a> Model for AccountLines<'a> {}

impl<'a> AccountLines<'a> {
    pub fn new(
        account: Cow<'a, str>,
        id: Option<Cow<'a, str>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        limit: Option<u16>,
        peer: Option<Cow<'a, str>>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            account,
            id,
            ledger_hash,
            ledger_index,
            limit,
            peer,
            marker,
            command: RequestMethod::AccountLines,
        }
    }
}
