use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::Amount;

/// Response format for the book_offers method, which retrieves a list of
/// offers, also known as the order book, between two currencies.
///
/// See Book Offers:
/// `<https://xrpl.org/book_offers.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BookOffers<'a> {
    /// Array of offer objects, each of which has the fields of an Offer object
    pub offers: Cow<'a, [BookOffer<'a>]>,
    /// The identifying hash of the ledger version that was used when
    /// retrieving this data, as requested.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version that was used when retrieving
    /// this data, as requested. Omitted if ledger_current_index is provided.
    pub ledger_index: Option<u32>,
    /// The ledger index of the current in-progress ledger version, which was
    /// used to retrieve this information. Omitted if ledger_index is provided.
    pub ledger_current_index: Option<u32>,
}

/// Represents an offer in the order book, with additional fields specific
/// to the book_offers response.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BookOffer<'a> {
    /// Bit-map of boolean flags enabled for this offer.
    pub flags: u32,
    /// The exchange rate, as the ratio taker_pays divided by taker_gets.
    /// For fairness, offers that have the same quality are automatically
    /// taken first-in, first-out.
    #[serde(rename = "quality")]
    pub quality: Cow<'a, str>,
    /// The unique ID of this offer.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The amount and type of currency being sold.
    pub taker_gets: Amount<'a>,
    /// The maximum amount of currency that the taker can get, given the
    /// funding status of the offer. Only included in partially-funded offers.
    pub taker_gets_funded: Option<Amount<'a>>,
    /// The amount and type of currency being bought.
    pub taker_pays: Amount<'a>,
    /// The maximum amount of currency that the taker would pay, given the
    /// funding status of the offer. Only included in partially-funded offers.
    pub taker_pays_funded: Option<Amount<'a>>,
    /// The account that placed this offer.
    pub account: Cow<'a, str>,
    /// Amount of the TakerGets currency the side placing the offer has
    /// available to be traded. (XRP is represented as drops; any other
    /// currency is represented as a decimal value.) If a trader has
    /// multiple offers in the same book, only the highest-ranked offer
    /// includes this field.
    pub owner_funds: Option<Cow<'a, str>>,
    /// The ID of the Offer Directory that links to this offer.
    pub book_directory: Cow<'a, str>,
    /// A hint indicating which page of the Offer Directory links to this
    /// object.
    pub book_node: Option<Cow<'a, str>>,
    /// Time after which this offer is considered expired.
    pub expiration: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_offers_deserialization() {
        let json = r#"{
            "ledger_current_index": 7035305,
            "offers": [
                {
                    "Account": "rM3X3QSr8icjTGpaF52dozhbT2BZSXJQYM",
                    "BookDirectory": "7E5F614417C2D0A7CEFEB73C4AA773ED5B078DE2B5771F6D55055E4C405218EB",
                    "BookNode": "0000000000000000",
                    "Flags": 0,
                    "LedgerEntryType": "Offer",
                    "OwnerNode": "0000000000000AE0",
                    "PreviousTxnID": "6956221794397C25A53647182E5C78A439766D600724074C99D78982E37599F1",
                    "PreviousTxnLgrSeq": 7022646,
                    "Sequence": 264542,
                    "TakerGets": {
                        "currency": "EUR",
                        "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                        "value": "17.90363633316433"
                    },
                    "TakerPays": {
                        "currency": "USD",
                        "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                        "value": "27.05340557506234"
                    },
                    "index": "96A9104BF3137131FF8310B9174F3B37170E2144C813CA2A1695DF2C5677E811",
                    "quality": "1.511056473200875"
                },
                {
                    "Account": "rhsxKNyN99q6vyYCTHNTC1TqWCeHr7PNgp",
                    "BookDirectory": "7E5F614417C2D0A7CEFEB73C4AA773ED5B078DE2B5771F6D5505DCAA8FE12000",
                    "BookNode": "0000000000000000",
                    "Flags": 131072,
                    "LedgerEntryType": "Offer",
                    "OwnerNode": "0000000000000001",
                    "PreviousTxnID": "8AD748CD489F7FF34FCD4FB73F77F1901E27A6EFA52CCBB0CCDAAB934E5E754D",
                    "PreviousTxnLgrSeq": 7007546,
                    "Sequence": 265,
                    "TakerGets": {
                        "currency": "EUR",
                        "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                        "value": "2.542743233917848"
                    },
                    "TakerPays": {
                        "currency": "USD",
                        "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
                        "value": "4.19552633596446"
                    },
                    "index": "7001797678E886E22D6DE11AF90DF1E08F4ADC21D763FAFB36AF66894D695235",
                    "quality": "1.65"
                }
            ]
        }"#;

        let book_offers: BookOffers = serde_json::from_str(json).unwrap();

        // Test main struct fields
        assert_eq!(book_offers.ledger_current_index, Some(7035305));
        assert_eq!(book_offers.ledger_hash, None);
        assert_eq!(book_offers.ledger_index, None);
        assert_eq!(book_offers.offers.len(), 2);

        // Test first offer
        let first_offer = &book_offers.offers[0];
        assert_eq!(first_offer.flags, 0);
        assert_eq!(first_offer.quality, "1.511056473200875");
        assert_eq!(
            first_offer.book_directory,
            "7E5F614417C2D0A7CEFEB73C4AA773ED5B078DE2B5771F6D55055E4C405218EB"
        );
        assert_eq!(first_offer.book_node, Some(Cow::from("0000000000000000")));
        assert_eq!(first_offer.account, "rM3X3QSr8icjTGpaF52dozhbT2BZSXJQYM");

        if let Amount::IssuedCurrencyAmount(amount) = &first_offer.taker_gets {
            assert_eq!(amount.currency, "EUR");
            assert_eq!(amount.issuer, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
            assert_eq!(amount.value, "17.90363633316433");
        } else {
            panic!("Expected IssuedCurrencyAmount for taker_gets");
        }

        if let Amount::IssuedCurrencyAmount(amount) = &first_offer.taker_pays {
            assert_eq!(amount.currency, "USD");
            assert_eq!(amount.issuer, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
            assert_eq!(amount.value, "27.05340557506234");
        } else {
            panic!("Expected IssuedCurrencyAmount for taker_pays");
        }

        // Test second offer
        let second_offer = &book_offers.offers[1];
        assert_eq!(second_offer.flags, 131072);
        assert_eq!(second_offer.quality, "1.65");
        assert_eq!(
            second_offer.book_directory,
            "7E5F614417C2D0A7CEFEB73C4AA773ED5B078DE2B5771F6D5505DCAA8FE12000"
        );
        assert_eq!(second_offer.book_node, Some(Cow::from("0000000000000000")));
        assert_eq!(second_offer.account, "rhsxKNyN99q6vyYCTHNTC1TqWCeHr7PNgp");

        if let Amount::IssuedCurrencyAmount(amount) = &second_offer.taker_gets {
            assert_eq!(amount.currency, "EUR");
            assert_eq!(amount.issuer, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
            assert_eq!(amount.value, "2.542743233917848");
        } else {
            panic!("Expected IssuedCurrencyAmount for taker_gets");
        }

        if let Amount::IssuedCurrencyAmount(amount) = &second_offer.taker_pays {
            assert_eq!(amount.currency, "USD");
            assert_eq!(amount.issuer, "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B");
            assert_eq!(amount.value, "4.19552633596446");
        } else {
            panic!("Expected IssuedCurrencyAmount for taker_pays");
        }
    }
}
