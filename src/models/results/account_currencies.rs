use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response from an account_currencies request, containing a list of
/// currencies that an accountn can send or receive, based on its trust lines.
///
/// See Account Currencies:
/// `<https://xrpl.org/account_currencies.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountCurrencies<'a> {
    /// The identifying hash of the ledger version used to retrieve this data,
    /// as hex.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version used to retrieve this data.
    pub ledger_index: u32,
    /// Array of Currency Codes for currencies that this account can receive.
    pub receive_currencies: Cow<'a, [Cow<'a, str>]>,
    /// Array of Currency Codes for currencies that this account can send.
    pub send_currencies: Cow<'a, [Cow<'a, str>]>,
    /// If true, this data comes from a validated ledger.
    pub validated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_currencies_deserialization() {
        let json = r#"{
            "ledger_index": 11775823,
            "receive_currencies": [
                "BTC",
                "CNY",
                "DYM",
                "EUR",
                "JOE",
                "MXN",
                "USD",
                "015841551A748AD2C1F76FF6ECB0CCCD00000000"
            ],
            "send_currencies": [
                "ASP",
                "BTC",
                "CHF",
                "CNY",
                "DYM",
                "EUR",
                "JOE",
                "JPY",
                "MXN",
                "USD"
            ],
            "status": "success",
            "validated": true
        }"#;

        let account_currencies: AccountCurrencies = serde_json::from_str(json).unwrap();

        assert_eq!(account_currencies.ledger_index, 11775823);
        assert_eq!(account_currencies.validated, true);

        // Test receive_currencies
        let expected_receive = [
            "BTC",
            "CNY",
            "DYM",
            "EUR",
            "JOE",
            "MXN",
            "USD",
            "015841551A748AD2C1F76FF6ECB0CCCD00000000",
        ];
        assert_eq!(
            account_currencies.receive_currencies.len(),
            expected_receive.len()
        );
        for (i, currency) in expected_receive.iter().enumerate() {
            assert_eq!(account_currencies.receive_currencies[i], *currency);
        }

        // Test send_currencies
        let expected_send = [
            "ASP", "BTC", "CHF", "CNY", "DYM", "EUR", "JOE", "JPY", "MXN", "USD",
        ];
        assert_eq!(
            account_currencies.send_currencies.len(),
            expected_send.len()
        );
        for (i, currency) in expected_send.iter().enumerate() {
            assert_eq!(account_currencies.send_currencies[i], *currency);
        }
    }
}
