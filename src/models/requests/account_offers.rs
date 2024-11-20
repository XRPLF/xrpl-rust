use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Marker, Request};

/// This request retrieves a list of offers made by a given account
/// that are outstanding as of a particular ledger version.
///
/// See Account Offers:
/// `<https://xrpl.org/account_offers.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountOffers<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// A unique identifier for the account, most commonly the
    /// account's Address.
    pub account: Cow<'a, str>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
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
    pub marker: Option<Marker<'a>>,
}

impl<'a> Model for AccountOffers<'a> {}

impl<'a> Request<'a> for AccountOffers<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> AccountOffers<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        limit: Option<u16>,
        strict: Option<bool>,
        marker: Option<Marker<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::AccountOffers,
                id,
            },
            account,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            limit,
            strict,
            marker,
        }
    }
}
