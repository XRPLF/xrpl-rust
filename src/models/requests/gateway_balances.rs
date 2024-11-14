use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

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
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// If true, only accept an address or public key for the
    /// account parameter. Defaults to false.
    pub strict: Option<bool>,
}

impl<'a> Model for GatewayBalances<'a> {}

impl<'a> Request<'a> for GatewayBalances<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> GatewayBalances<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        hotwallet: Option<Vec<Cow<'a, str>>>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        strict: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::GatewayBalances,
                id,
            },
            account,
            strict,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            hotwallet,
        }
    }
}
