use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{currency::Currency, requests::RequestMethod, Model};

use super::{CommonFields, Request};

/// The book_offers method retrieves a list of offers, also known
/// as the order book, between two currencies.
///
/// See Book Offers:
/// `<https://xrpl.org/book_offers.html#book_offers>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BookOffers<'a> {
    /// The common fields shared by all requests.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a>,
    /// Specification of which currency the account taking
    /// the offer would receive, as an object with currency
    /// and issuer fields (omit issuer for XRP),
    /// like currency amounts.
    pub taker_gets: Currency<'a>,
    /// Specification of which currency the account taking
    /// the offer would pay, as an object with currency and
    /// issuer fields (omit issuer for XRP),
    /// like currency amounts.
    pub taker_pays: Currency<'a>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<Cow<'a, str>>,
    /// If provided, the server does not provide more than
    /// this many offers in the results. The total number of
    /// results returned may be fewer than the limit,
    /// because the server omits unfunded offers.
    pub limit: Option<u16>,
    /// The Address of an account to use as a perspective.
    /// Unfunded offers placed by this account are always
    /// included in the response. (You can use this to look
    /// up your own orders to cancel them.)
    pub taker: Option<Cow<'a, str>>,
}

impl<'a> Model for BookOffers<'a> {}

impl<'a> Request<'a> for BookOffers<'a> {
    fn get_common_fields(&self) -> &CommonFields<'a> {
        &self.common_fields
    }

    fn get_common_fields_mut(&mut self) -> &mut CommonFields<'a> {
        &mut self.common_fields
    }
}

impl<'a> BookOffers<'a> {
    pub fn new(
        id: Option<Cow<'a, str>>,
        taker_gets: Currency<'a>,
        taker_pays: Currency<'a>,
        ledger_hash: Option<Cow<'a, str>>,
        ledger_index: Option<Cow<'a, str>>,
        limit: Option<u16>,
        taker: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                command: RequestMethod::BookOffers,
                id,
            },
            taker_gets,
            taker_pays,
            ledger_hash,
            ledger_index,
            limit,
            taker,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::currency::{Currency, IssuedCurrency, XRP};

    use super::BookOffers;

    #[test]
    fn test_serde() {
        let req = BookOffers::new(
            None,
            Currency::IssuedCurrency(IssuedCurrency::new("EUR".into(), "rTestIssuer".into())),
            Currency::XRP(XRP::new()),
            None,
            None,
            None,
            None,
        );
        let serialized = serde_json::to_string(&req).unwrap();

        let deserialized: BookOffers = serde_json::from_str(&serialized).unwrap();

        assert_eq!(req, deserialized);
    }
}
