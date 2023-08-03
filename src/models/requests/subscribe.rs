use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum_macros::Display;

use crate::models::{currency::Currency, default_false, requests::RequestMethod, Model};

/// Format for elements in the `books` array for Subscribe only.
///
/// See Subscribe:
/// `<https://xrpl.org/subscribe.html#subscribe>`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all(serialize = "PascalCase", deserialize = "snake_case"))]
pub struct SubscribeBook<'a> {
    pub taker_gets: Currency<'a>,
    pub taker_pays: Currency<'a>,
    pub taker: &'a str,
    #[serde(default = "default_false")]
    pub snapshot: Option<bool>,
    #[serde(default = "default_false")]
    pub both: Option<bool>,
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
    /// The unique request id.
    pub id: Option<&'a str>,
    /// Array of objects defining order books  to monitor for
    /// updates, as detailed below.
    pub books: Option<Vec<SubscribeBook<'a>>>,
    /// Array of string names of generic streams to subscribe to.
    pub streams: Option<Vec<StreamParameter>>,
    /// Array with the unique addresses of accounts to monitor
    /// for validated transactions. The addresses must be in the
    /// XRP Ledger's base58 format. The server sends a notification
    /// for any transaction that affects at least one of these accounts.
    pub accounts: Option<Vec<&'a str>>,
    /// Like accounts, but include transactions that are not
    /// yet finalized.
    pub accounts_proposed: Option<Vec<&'a str>>,
    /// (Optional for Websocket; Required otherwise) URL where the server
    /// sends a JSON-RPC callbacks for each event. Admin-only.
    pub url: Option<&'a str>,
    /// Username to provide for basic authentication at the callback URL.
    pub url_username: Option<&'a str>,
    /// Password to provide for basic authentication at the callback URL.
    pub url_password: Option<&'a str>,
    /// The request method.
    // #[serde(skip_serializing)]
    #[serde(default = "RequestMethod::subscribe")]
    pub command: RequestMethod,
}

impl<'a> Default for Subscribe<'a> {
    fn default() -> Self {
        Subscribe {
            id: None,
            books: None,
            streams: None,
            accounts: None,
            accounts_proposed: None,
            url: None,
            url_username: None,
            url_password: None,
            command: RequestMethod::Subscribe,
        }
    }
}

impl<'a> Model for Subscribe<'a> {}

impl<'a> Subscribe<'a> {
    pub fn new(
        id: Option<&'a str>,
        books: Option<Vec<SubscribeBook<'a>>>,
        streams: Option<Vec<StreamParameter>>,
        accounts: Option<Vec<&'a str>>,
        accounts_proposed: Option<Vec<&'a str>>,
        url: Option<&'a str>,
        url_username: Option<&'a str>,
        url_password: Option<&'a str>,
    ) -> Self {
        Self {
            id,
            books,
            streams,
            accounts,
            accounts_proposed,
            url,
            url_username,
            url_password,
            command: RequestMethod::Subscribe,
        }
    }
}
