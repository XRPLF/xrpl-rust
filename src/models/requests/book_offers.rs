use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{currency::Currency, requests::RequestMethod, Model};

/// The book_offers method retrieves a list of offers, also known
/// as the order book, between two currencies.
///
/// See Book Offers:
/// `<https://xrpl.org/book_offers.html#book_offers>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BookOffers<'a> {
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
    /// The unique request id.
    pub id: Option<&'a str>,
    /// A 20-byte hex string for the ledger version to use.
    pub ledger_hash: Option<&'a str>,
    /// The ledger index of the ledger to use, or a shortcut
    /// string to choose a ledger automatically.
    pub ledger_index: Option<&'a str>,
    /// If provided, the server does not provide more than
    /// this many offers in the results. The total number of
    /// results returned may be fewer than the limit,
    /// because the server omits unfunded offers.
    pub limit: Option<u16>,
    /// The Address of an account to use as a perspective.
    /// Unfunded offers placed by this account are always
    /// included in the response. (You can use this to look
    /// up your own orders to cancel them.)
    pub taker: Option<&'a str>,
    /// The request method.
    #[serde(default = "RequestMethod::book_offers")]
    pub command: RequestMethod,
}

impl<'a> Default for BookOffers<'a> {
    fn default() -> Self {
        BookOffers {
            taker_gets: Default::default(),
            taker_pays: Default::default(),
            id: None,
            ledger_hash: None,
            ledger_index: None,
            limit: None,
            taker: None,
            command: RequestMethod::BookOffers,
        }
    }
}

impl<'a> Model for BookOffers<'a> {}

impl<'a> BookOffers<'a> {
    fn new(
        taker_gets: Currency<'a>,
        taker_pays: Currency<'a>,
        id: Option<&'a str>,
        ledger_hash: Option<&'a str>,
        ledger_index: Option<&'a str>,
        limit: Option<u16>,
        taker: Option<&'a str>,
    ) -> Self {
        Self {
            taker_gets,
            taker_pays,
            id,
            ledger_hash,
            ledger_index,
            limit,
            taker,
            command: RequestMethod::BookOffers,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::currency::{Currency, IssuedCurrency, XRP};

    use super::BookOffers;

    #[test]
    fn test_serde() {
        let req = BookOffers {
            taker_gets: Currency::IssuedCurrency(IssuedCurrency::new(
                "EUR".into(),
                "rTestIssuer".into(),
            )),
            taker_pays: Currency::XRP(XRP::new()),
            ..Default::default()
        };
        let req_as_string = serde_json::to_string(&req).unwrap();
        let req_json = req_as_string.as_str();
        let expected_json = r#"{"taker_gets":{"currency":"EUR","issuer":"rTestIssuer"},"taker_pays":{"currency":"XRP"},"command":"book_offers"}"#;
        let deserialized_req: BookOffers = serde_json::from_str(req_json).unwrap();

        assert_eq!(req_json, expected_json);
        assert_eq!(req, deserialized_req);
        assert_eq!(Currency::XRP(XRP::new()), deserialized_req.taker_pays);
    }
}
