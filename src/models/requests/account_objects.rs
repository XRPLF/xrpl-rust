use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::{Display, EnumString};

use crate::models::{requests::RequestMethod, Model};

use super::{CommonFields, LedgerIndex, LookupByLedgerRequest, Marker, Request};

/// Represents the object types that an AccountObjects
/// Request can ask for.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum AccountObjectType {
    Check,
    DepositPreauth,
    Escrow,
    Offer,
    PaymentChannel,
    SignerList,
    State,
    Ticket,
}

/// This request returns the raw ledger format for all objects
/// owned by an account. For a higher-level view of an account's
/// trust lines and balances, see AccountLines Request instead.
///
/// See Account Objects:
/// `<https://xrpl.org/account_objects.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AccountObjects<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// A unique identifier for the account, most commonly the
    /// account's address.
    pub account: Cow<'a, str>,
    /// The unique identifier of a ledger.
    #[serde(flatten)]
    pub ledger_lookup: Option<LookupByLedgerRequest<'a>>,
    /// If included, filter results to include only this type
    /// of ledger object. The valid types are: check, deposit_preauth,
    /// escrow, offer, payment_channel, signer_list, ticket,
    /// and state (trust line).
    pub r#type: Option<AccountObjectType>,
    /// If true, the response only includes objects that would block
    /// this account from being deleted. The default is false.
    pub deletion_blockers_only: Option<bool>,
    /// The maximum number of objects to include in the results.
    /// Must be within the inclusive range 10 to 400 on non-admin
    /// connections. The default is 200.
    pub limit: Option<u16>,
    /// Value from a previous paginated response. Resume retrieving
    /// data where that response left off.
    pub marker: Option<Marker<'a>>,
}

impl<'a> Model for AccountObjects<'a> {}

impl<'a> Request<'a> for AccountObjects<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> AccountObjects<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        account: Cow<'a, str>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<LedgerIndex<'a>>,
        r#type: Option<AccountObjectType>,
        deletion_blockers_only: Option<bool>,
        limit: Option<u16>,
        marker: Option<Marker<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::AccountObjects,
                id,
            },
            account,
            ledger_lookup: Some(LookupByLedgerRequest {
                ledger_hash,
                ledger_index,
            }),
            r#type,
            deletion_blockers_only,
            limit,
            marker,
        }
    }
}
