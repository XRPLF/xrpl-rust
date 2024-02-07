use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// This request calculates the total balances issued by a
/// given account, optionally excluding amounts held by
/// operational addresses.
///
/// See Gateway Balances:
/// `<https://xrpl.org/gateway_balances.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct GatewayBalances<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// The Address to check. This should be the issuing address.
    pub account: Cow<'a, str>,
    /// An operational address to exclude from the balances
    /// issued, or an array of such addresses.
    pub hotwallet: Option<Vec<Cow<'a, str>>>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version to use, or a
    /// shortcut string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// If true, only accept an address or public key for the
    /// account parameter. Defaults to false.
    pub strict: Option<bool>,
}

impl<'a> Model for GatewayBalances<'a> {}

impl<'a> Request for GatewayBalances<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> GatewayBalances<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        hotwallet: Option<Vec<Cow<'a, str>>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        strict: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::GatewayBalances,
                id,
            },
            account,
            strict,
            ledger_hash,
            ledger_index,
            hotwallet,
        }
    }
}
