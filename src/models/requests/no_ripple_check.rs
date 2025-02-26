use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::Display;

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Request};

/// Enum representing the options for the address role in
/// a NoRippleCheckRequest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "role")]
#[derive(Default)]
pub enum NoRippleCheckRole {
    #[default]
    User,
    Gateway,
}

/// This request provides a quick way to check the status of
/// the Default Ripple field for an account and the No Ripple
/// flag of its trust lines, compared with the recommended
/// settings.
///
/// See No Ripple Check:
/// `<https://xrpl.org/noripple_check.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NoRippleCheck<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// A unique identifier for the account, most commonly the
    /// account's address.
    pub account: Cow<'a, str>,
    /// Whether the address refers to a gateway or user.
    /// Recommendations depend on the role of the account.
    /// Issuers must have Default Ripple enabled and must disable
    /// No Ripple on all trust lines. Users should have Default Ripple
    /// disabled, and should enable No Ripple on all trust lines.
    pub role: NoRippleCheckRole,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// The maximum number of trust line problems to include in the
    /// results. Defaults to 300.
    pub limit: Option<u16>,
    /// If true, include an array of suggested transactions, as JSON
    /// objects, that you can sign and submit to fix the problems.
    /// Defaults to false.
    pub transactions: Option<bool>,
}

impl<'a> Model for NoRippleCheck<'a> {}

impl<'a> Request<'a> for NoRippleCheck<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> NoRippleCheck<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        role: NoRippleCheckRole,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        limit: Option<u16>,
        transactions: Option<bool>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::NoRippleCheck,
                id,
            },
            account,
            role,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            transactions,
            limit,
        }
    }
}
