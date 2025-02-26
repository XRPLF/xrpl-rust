use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::{requests::Marker, Amount};

/// Response from an account_offers request, containing a list of offers made
/// by a given account that are outstanding as of a particular ledger version.
///
/// See Account Offers:
/// `<https://xrpl.org/account_offers.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountOffers<'a> {
    /// Unique Address identifying the account that made the offers
    pub account: Cow<'a, str>,
    /// Array of objects, where each object represents an offer made by this
    /// account that is outstanding as of the requested ledger version. If the
    /// number of offers is large, only returns up to limit at a time.
    pub offers: Cow<'a, [OfferObject<'a>]>,
    /// The identifying hash of the ledger version that was used when
    /// retrieving this data.
    /// May be omitted.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version that was used when retrieving
    /// this data, as requested. Omitted if ledger_current_index is provided
    /// instead.
    pub ledger_index: Option<u32>,
    /// The ledger index of the current in-progress ledger version, which was
    /// used when retrieving this data. Omitted if ledger_hash or ledger_index
    /// is provided.
    pub ledger_current_index: Option<u32>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off. Omitted when
    /// there are no pages of information after this one.
    pub marker: Option<Marker<'a>>,
}

/// Represents a single offer object in the account_offers response.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OfferObject<'a> {
    /// Options set for this offer entry as bit-flags.
    pub flags: u32,
    /// Sequence number of the transaction that created this entry.
    /// Transaction sequence numbers are relative to accounts.
    pub seq: u32,
    /// The amount the account accepting the offer receives, as a String
    /// representing an amount in XRP, or a currency specification object.
    pub taker_gets: Cow<'a, str>,
    /// The amount the account accepting the offer provides, as a String
    /// representing an amount in XRP, or a currency specification object.
    pub taker_pays: Amount<'a>,
    /// The exchange rate of the offer, as the ratio of the original
    /// taker_pays divided by the original taker_gets. When executing offers,
    /// the offer with the most favorable (lowest) quality is consumed first;
    /// offers with the same quality are executed from oldest to newest.
    pub quality: Option<Cow<'a, str>>,
    /// A time after which this offer is considered unfunded, as the number
    /// of seconds since the Ripple Epoch.
    pub expiration: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_offers_deserialization() {
        let json = r#"{
            "account": "rpP2JgiMyTF5jR5hLG3xHCPi1knBb1v9cM",
            "ledger_current_index": 18539596,
            "offers": [{
                "flags": 0,
                "quality": "0.000000007599140009999998",
                "seq": 6578020,
                "taker_gets": "29740867287",
                "taker_pays": {
                    "currency": "USD",
                    "issuer": "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q",
                    "value": "226.0050145327418"
                }
            }, {
                "flags": 0,
                "quality": "7989247009094510e-27",
                "seq": 6572128,
                "taker_gets": "2361918758",
                "taker_pays": {
                    "currency": "XAU",
                    "issuer": "rrh7rf1gV2pXAoqA8oYbpHd8TKv5ZQeo67",
                    "value": "0.01886995237307572"
                }
            }, {
                "flags": 0,
                "quality": "0.00000004059594001318974",
                "seq": 6576905,
                "taker_gets": "3892952574",
                "taker_pays": {
                    "currency": "CNY",
                    "issuer": "rKiCet8SdvWxPXnAgYarFUXMh1zCPz432Y",
                    "value": "158.0380691682966"
                }
            }],
            "status": "success",
            "validated": false
        }"#;

        let account_offers: AccountOffers = serde_json::from_str(json).unwrap();

        // Test main struct fields
        assert_eq!(account_offers.account, "rpP2JgiMyTF5jR5hLG3xHCPi1knBb1v9cM");
        assert_eq!(account_offers.ledger_current_index, Some(18539596));
        assert_eq!(account_offers.offers.len(), 3);

        // Test first offer
        let first_offer = &account_offers.offers[0];
        assert_eq!(first_offer.flags, 0);
        assert_eq!(
            first_offer.quality.as_ref().unwrap(),
            "0.000000007599140009999998"
        );
        assert_eq!(first_offer.seq, 6578020);
        assert_eq!(first_offer.taker_gets, "29740867287");

        if let Amount::IssuedCurrencyAmount(taker_pays) = &first_offer.taker_pays {
            assert_eq!(taker_pays.currency, "USD");
            assert_eq!(taker_pays.issuer, "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q");
            assert_eq!(taker_pays.value, "226.0050145327418");
        } else {
            panic!("Expected IssuedCurrencyAmount for first offer taker_pays");
        }

        // Test second offer
        let second_offer = &account_offers.offers[1];
        assert_eq!(second_offer.flags, 0);
        assert_eq!(
            second_offer.quality.as_ref().unwrap(),
            "7989247009094510e-27"
        );
        assert_eq!(second_offer.seq, 6572128);
        assert_eq!(second_offer.taker_gets, "2361918758");

        if let Amount::IssuedCurrencyAmount(taker_pays) = &second_offer.taker_pays {
            assert_eq!(taker_pays.currency, "XAU");
            assert_eq!(taker_pays.issuer, "rrh7rf1gV2pXAoqA8oYbpHd8TKv5ZQeo67");
            assert_eq!(taker_pays.value, "0.01886995237307572");
        } else {
            panic!("Expected IssuedCurrencyAmount for second offer taker_pays");
        }
    }
}
