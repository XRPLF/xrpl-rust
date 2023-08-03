use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

/// This request calculates the total balances issued by a
/// given account, optionally excluding amounts held by
/// operational addresses.
///
/// See Gateway Balances:
/// `<https://xrpl.org/gateway_balances.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct GatewayBalances<'a> {
    /// The Address to check. This should be the issuing address.
    pub account: &'a str,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// If true, only accept an address or public key for the
    /// account parameter. Defaults to false.
    pub strict: Option<bool>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger version to use, or a
    /// shortcut string to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// An operational address to exclude from the balances
    /// issued, or an array of such addresses.
    pub hotwallet: Option<Vec<&'a str>>,
    /// The request method.
    #[serde(default = "RequestMethod::deposit_authorization")]
    pub command: RequestMethod,
}

impl<'a> Default for GatewayBalances<'a> {
    fn default() -> Self {
        GatewayBalances {
            account: "",
            id: None,
            strict: None,
            ledger_hash: None,
            ledger_index: None,
            hotwallet: None,
            command: RequestMethod::GatewayBalances,
        }
    }
}

impl<'a> Model for GatewayBalances<'a> {}

impl<'a> GatewayBalances<'a> {
    pub fn new(
        account: &'a str,
        id: Option<&'a str>,
        strict: Option<bool>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        hotwallet: Option<Vec<&'a str>>,
    ) -> Self {
        Self {
            account,
            id,
            strict,
            ledger_hash,
            ledger_index,
            hotwallet,
            command: RequestMethod::GatewayBalances,
        }
    }
}
