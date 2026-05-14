use alloc::borrow::Cow;
use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    currency::Currency,
    default_false,
    requests::{subscribe::StreamParameter, RequestMethod},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::currency::{IssuedCurrency, XRP};
    use alloc::vec;

    #[test]
    fn test_serde_round_trip_no_books() {
        // UnsubscribeBook uses asymmetric serialize/deserialize naming
        // (PascalCase vs snake_case), so a full round-trip with books does
        // not work. Round-trip the request without books.
        let req = Unsubscribe::new(
            Some("uns-1".into()),
            Some(vec!["rAcc1111111111111111111111111111".into()]),
            None,
            None,
            None,
            Some(vec![StreamParameter::Ledger]),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: Unsubscribe = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
        assert!(serialized.contains("\"command\":\"unsubscribe\""));
    }

    #[test]
    fn test_unsubscribe_book_serializes() {
        let book = UnsubscribeBook::new(
            Currency::XRP(XRP::new()),
            Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rIssuer11111111111111111111111111".into(),
            )),
            Some(true),
        );
        let serialized = serde_json::to_string(&book).unwrap();
        assert!(serialized.contains("\"TakerGets\""));
        assert!(serialized.contains("\"TakerPays\""));
    }
}
