use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This request retrieves from the ledger a list of
/// transactions that involved the specified account.
///
/// See Account Tx:
/// `<https://xrpl.org/account_tx.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountTx<'a> {
    /// A unique identifier for the account, most commonly the
    /// account's address.
    pub account: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// Use to look for transactions from a single ledger only.
    pub ledger_hash: Option<&'a str>,
    /// Use to look for transactions from a single ledger only.
    pub ledger_index: Option<&'a str>,
    /// Defaults to false. If set to true, returns transactions
    /// as hex strings instead of JSON.
    pub binary: Option<bool>,
    /// Defaults to false. If set to true, returns values indexed
    /// with the oldest ledger first. Otherwise, the results are
    /// indexed with the newest ledger first.
    /// (Each page of results may not be internally ordered, but
    /// the pages are overall ordered.)
    pub forward: Option<bool>,
    /// Use to specify the earliest ledger to include transactions
    /// from. A value of -1 instructs the server to use the earliest
    /// validated ledger version available.
    pub ledger_index_min: Option<u32>,
    /// Use to specify the most recent ledger to include transactions
    /// from. A value of -1 instructs the server to use the most
    /// recent validated ledger version available.
    pub ledger_index_max: Option<u32>,
    /// Default varies. Limit the number of transactions to retrieve.
    /// The server is not required to honor this value.
    pub limit: Option<u16>,
    /// Value from a previous paginated response. Resume retrieving
    /// data where that response left off. This value is stable even
    /// if there is a change in the server's range of available
    /// ledgers.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::account_tx")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountTx<'a> {
    fn default() -> Self {
        AccountTx {
            account: "",
            id: None,
            ledger_hash: None,
            ledger_index: None,
            binary: None,
            forward: None,
            ledger_index_min: None,
            ledger_index_max: None,
            limit: None,
            marker: None,
            command: RequestMethod::AccountTx,
        }
    }
}

impl<'a> Model for AccountTx<'a> {}

impl<'a> AccountTx<'a> {
    fn new(
        account: &'a str,
        id: Option<&'a str>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        binary: Option<bool>,
        forward: Option<bool>,
        ledger_index_min: Option<u32>,
        ledger_index_max: Option<u32>,
        limit: Option<u16>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            account,
            id,
            ledger_hash,
            ledger_index,
            binary,
            forward,
            ledger_index_min,
            ledger_index_max,
            limit,
            marker,
            command: RequestMethod::AccountTx,
        }
    }
}
