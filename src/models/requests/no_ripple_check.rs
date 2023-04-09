use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::Display;

use crate::models::{requests::RequestMethod, Model};

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
    /// A unique identifier for the account, most commonly the
    /// account's address.
    pub account: &'a str,
    /// Whether the address refers to a gateway or user.
    /// Recommendations depend on the role of the account.
    /// Issuers must have Default Ripple enabled and must disable
    /// No Ripple on all trust lines. Users should have Default Ripple
    /// disabled, and should enable No Ripple on all trust lines.
    pub role: NoRippleCheckRole,
    /// The unique request id.
    pub id: Option<&'a str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut string
    /// to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// If true, include an array of suggested transactions, as JSON
    /// objects, that you can sign and submit to fix the problems.
    /// Defaults to false.
    pub transactions: Option<bool>,
    /// The maximum number of trust line problems to include in the
    /// results. Defaults to 300.
    pub limit: Option<u16>,
    /// The request method.
    #[serde(default = "RequestMethod::no_ripple_check")]
    pub command: RequestMethod,
}

impl<'a> Default for NoRippleCheck<'a> {
    fn default() -> Self {
        NoRippleCheck {
            account: "",
            role: Default::default(),
            id: None,
            ledger_hash: None,
            ledger_index: None,
            transactions: None,
            limit: None,
            command: RequestMethod::NoRippleCheck,
        }
    }
}

impl<'a> Model for NoRippleCheck<'a> {}

impl<'a> NoRippleCheck<'a> {
    fn new(
        account: &'a str,
        role: NoRippleCheckRole,
        id: Option<&'a str>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        transactions: Option<bool>,
        limit: Option<u16>,
    ) -> Self {
        Self {
            account,
            role,
            id,
            ledger_hash,
            ledger_index,
            transactions,
            limit,
            command: RequestMethod::NoRippleCheck,
        }
    }
}
