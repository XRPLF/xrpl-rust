use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    currency::Currency,
    default_false,
    requests::{RequestMethod, StreamParameter},
    Model,
};

use super::{CommonFields, Request};

/// Format for elements in the `books` array for Unsubscribe only.
///
/// See Unsubscribe:
/// `<https://xrpl.org/unsubscribe.html>`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all(serialize = "PascalCase", deserialize = "snake_case"))]
pub struct UnsubscribeBook<'a> {
    pub taker_gets: Currency<'a>,
    pub taker_pays: Currency<'a>,
    #[serde(default = "default_false")]
    pub both: Option<bool>,
}

/// The unsubscribe command tells the server to stop
/// sending messages for a particular subscription or set
/// of subscriptions.
///
/// Note: WebSocket API only.
///
/// See Unsubscribe:
/// `<https://xrpl.org/unsubscribe.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Unsubscribe<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Array of unique account addresses to stop receiving updates
    /// for, in the XRP Ledger's base58 format. (This only stops
    /// those messages if you previously subscribed to those accounts
    /// specifically. You cannot use this to filter accounts out of
    /// the general transactions stream.)
    pub accounts: Option<Vec<Cow<'a, str>>>,
    /// Like accounts, but for accounts_proposed subscriptions that
    /// included not-yet-validated transactions.
    pub accounts_proposed: Option<Vec<Cow<'a, str>>>,
    /// Array of objects defining order books to unsubscribe
    /// from, as explained below.
    pub books: Option<Vec<UnsubscribeBook<'a>>>,
    #[serde(skip_serializing)]
    pub broken: Option<Cow<'a, str>>,
    /// Array of string names of generic streams to unsubscribe
    /// from, including ledger, server, transactions,
    /// and transactions_proposed.
    pub streams: Option<Vec<StreamParameter>>,
}

impl<'a> Model for Unsubscribe<'a> {}

impl<'a> Request<'a> for Unsubscribe<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> Unsubscribe<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        accounts: Option<Vec<Cow<'a, str>>>,
        accounts_proposed: Option<Vec<Cow<'a, str>>>,
        books: Option<Vec<UnsubscribeBook<'a>>>,
        broken: Option<Cow<'a, str>>,
        streams: Option<Vec<StreamParameter>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Unsubscribe,
                id,
            },
            books,
            streams,
            accounts,
            accounts_proposed,
            broken,
        }
    }
}
