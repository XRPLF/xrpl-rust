use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::Display;

use crate::models::{currency::Currency, default_false, requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// Format for elements in the `books` array for Subscribe only.
///
/// See Subscribe:
/// `<https://xrpl.org/subscribe.html#subscribe>`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all(serialize = "PascalCase", deserialize = "snake_case"))]
pub struct SubscribeBook<'a> {
    pub taker: Cow<'a, str>,
    pub taker_gets: Currency<'a>,
    pub taker_pays: Currency<'a>,
    #[serde(default = "default_false")]
    pub both: Option<bool>,
    #[serde(default = "default_false")]
    pub snapshot: Option<bool>,
}

/// Represents possible values of the streams query param
/// for subscribe.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum StreamParameter {
    Consensus,
    Ledger,
    Manifests,
    PeerStatus,
    Transactions,
    TransactionsProposed,
    Server,
    Validations,
}

/// The subscribe method requests periodic notifications
/// from the server when certain events happen.
///
/// Note: WebSocket API only.
///
/// See Subscribe:
/// `<https://xrpl.org/subscribe.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Subscribe<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Array with the unique addresses of accounts to monitor
    /// for validated transactions. The addresses must be in the
    /// XRP Ledger's base58 format. The server sends a notification
    /// for any transaction that affects at least one of these accounts.
    pub accounts: Option<Vec<Cow<'a, str>>>,
    /// Like accounts, but include transactions that are not
    /// yet finalized.
    pub accounts_proposed: Option<Vec<Cow<'a, str>>>,
    /// Array of objects defining order books  to monitor for
    /// updates, as detailed below.
    pub books: Option<Vec<SubscribeBook<'a>>>,
    /// Array of string names of generic streams to subscribe to.
    pub streams: Option<Vec<StreamParameter>>,
    /// (Optional for Websocket; Required otherwise) URL where the server
    /// sends a JSON-RPC callbacks for each event. Admin-only.
    pub url: Option<Cow<'a, str>>,
    /// Password to provide for basic authentication at the callback URL.
    pub url_password: Option<Cow<'a, str>>,
    /// Username to provide for basic authentication at the callback URL.
    pub url_username: Option<Cow<'a, str>>,
}

impl<'a> Model for Subscribe<'a> {}

impl<'a> Request for Subscribe<'a> {
    fn get_command(&self) -> RequestMethod {
        self.common_fields.command.clone()
    }
}

impl<'a> Subscribe<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        accounts: Option<Vec<Cow<'a, str>>>,
        accounts_proposed: Option<Vec<Cow<'a, str>>>,
        books: Option<Vec<SubscribeBook<'a>>>,
        streams: Option<Vec<StreamParameter>>,
        url: Option<Cow<'a, str>>,
        url_password: Option<Cow<'a, str>>,
        url_username: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::Subscribe,
                id,
            },
            books,
            streams,
            accounts,
            accounts_proposed,
            url,
            url_username,
            url_password,
        }
    }
}
