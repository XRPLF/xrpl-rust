use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{default_false, requests::RequestMethod, Model};

/// This request retrieves a list of currencies that an account
/// can send or receive, based on its trust lines. This is not
/// a thoroughly confirmed list, but it can be used to populate
/// user interfaces.
///
/// See Account Currencies:
/// `<https://xrpl.org/account_currencies.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountCurrencies<'a> {
    /// A unique identifier for the account, most commonly
    /// the account's Address.
    pub account: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// If true, then the account field only accepts a public
    /// key or XRP Ledger address. Otherwise, account can be
    /// a secret or passphrase (not recommended).
    /// The default is false.
    #[serde(default = "default_false")]
    pub strict: Option<bool>,
    /// The request method.
    #[serde(default = "RequestMethod::account_currencies")]
    pub command: RequestMethod,
}

impl<'a> Default for AccountCurrencies<'a> {
    fn default() -> Self {
        AccountCurrencies {
            account: "",
            id: None,
            ledger_hash: None,
            ledger_index: None,
            strict: None,
            command: RequestMethod::AccountCurrencies,
        }
    }
}

impl<'a> Model for AccountCurrencies<'a> {}

impl<'a> AccountCurrencies<'a> {
    fn new(
        account: &'a str,
        id: Option<&'a str>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        strict: Option<bool>,
    ) -> Self {
        Self {
            account,
            id,
            ledger_hash,
            ledger_index,
            strict,
            command: RequestMethod::AccountCurrencies,
        }
    }
}
