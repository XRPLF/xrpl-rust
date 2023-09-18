use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This request retrieves a list of offers made by a given account
/// that are outstanding as of a particular ledger version.
///
/// See Account Offers:
/// `<https://xrpl.org/account_offers.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountOffers<'a> {
    /// A unique identifier for the account, most commonly the
    /// account's Address.
    pub account: Cow<'a, str>,
    /// The unique request id.
    pub id: Option<Cow<'a, str>>,
    /// A 20-byte hex string identifying the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or "current",
    /// "closed", or "validated" to select a ledger dynamically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// Limit the number of transactions to retrieve. The server is
    /// not required to honor this value. Must be within the inclusive
    /// range 10 to 400.
    pub limit: Option<u16>,
    /// If true, then the account field only accepts a public key or
    /// XRP Ledger address. Otherwise, account can be a secret or
    /// passphrase (not recommended). The default is false.
    pub strict: Option<bool>,
    /// Value from a previous paginated response. Resume retrieving
    /// data where that response left off.
    pub marker: Option<u32>,
    /// The request method.
    #[serde(default = "RequestMethod::account_offers")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountOffers<'a> {
    fn default() -> Self {
        AccountOffers {
            account: "".into(),
            id: None,
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            strict: None,
            marker: None,
            command: RequestMethod::AccountOffers,
        }
    }
}

impl<'a> Model for AccountOffers<'a> {}

impl<'a> AccountOffers<'a> {
    pub fn new(
        account: Cow<'a, str>,
        id: Option<Cow<'a, str>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        limit: Option<u16>,
        strict: Option<bool>,
        marker: Option<u32>,
    ) -> Self {
        Self {
            account,
            id,
            ledger_hash,
            ledger_index,
            limit,
            strict,
            marker,
            command: RequestMethod::AccountOffers,
        }
    }
}
