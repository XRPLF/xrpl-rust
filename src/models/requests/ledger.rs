use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// Retrieve information about the public ledger.
///
/// See Ledger Data:
/// `<https://xrpl.org/ledger.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ledger<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Admin required. If true, return information on accounts
    /// in the ledger. Ignored if you did not specify a ledger
    /// version. Defaults to false. Caution: This returns a very
    /// large amount of data!
    pub accounts: Option<bool>,
    /// If true, and transactions and expand are both also true,
    /// return transaction information in binary format
    /// (hexadecimal string) instead of JSON format.
    pub binary: Option<bool>,
    /// Provide full JSON-formatted information for
    /// transaction/account information instead of only hashes.
    /// Defaults to false. Ignored unless you request transactions,
    /// accounts, or both.
    pub expand: Option<bool>,
    /// Admin required. If true, return full information on
    /// the entire ledger. Ignored if you did not specify a
    /// ledger version. Defaults to false. (Equivalent to
    /// enabling transactions, accounts, and expand.)
    /// Caution: This is a very large amount of data -- on
    /// the order of several hundred megabytes!
    pub full: Option<bool>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// If true, include owner_funds field in the metadata of
    /// OfferCreate transactions in the response. Defaults to
    /// false. Ignored unless transactions are included and
    /// expand is true.
    pub owner_funds: Option<bool>,
    /// If true, and the command is requesting the current ledger,
    /// includes an array of queued transactions in the results.
    pub queue: Option<bool>,
    /// If true, return information on transactions in the
    /// specified ledger version. Defaults to false. Ignored if
    /// you did not specify a ledger version.
    pub transactions: Option<bool>,
}

impl<'a> Model for Ledger<'a> {}

impl<'a> Request<'a> for Ledger<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Ledger<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        accounts: Option<bool>,
        binary: Option<bool>,
        expand: Option<bool>,
        full: Option<bool>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        owner_funds: Option<bool>,
        queue: Option<bool>,
        transactions: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Ledger,
                id,
            },
            ledger_hash,
            ledger_index,
            full,
            accounts,
            transactions,
            expand,
            owner_funds,
            binary,
            queue,
        }
    }
}
