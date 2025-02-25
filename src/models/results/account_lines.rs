use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::models::requests::Marker;

/// Response from an account_lines request, containing information about an
/// account's trust lines.
///
/// See Account Lines:
/// `<https://xrpl.org/account_lines.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountLines<'a> {
    /// Unique Address of the account this request corresponds to. This is
    /// the "perspective account" for purpose of the trust lines.
    pub account: Cow<'a, str>,
    /// Array of trust line objects. If the number of trust lines is large,
    /// only returns up to the limit at a time.
    pub lines: Cow<'a, [TrustLine<'a>]>,
    /// (Omitted if ledger_hash or ledger_index provided) The ledger index of
    /// the current open ledger, which was used when retrieving this
    /// information.
    pub ledger_current_index: Option<u32>,
    /// (Omitted if ledger_current_index provided instead) The ledger index
    /// of the ledger version that was used when retrieving this data.
    pub ledger_index: Option<u32>,
    /// (May be omitted) The identifying hash the ledger version that was
    /// used when retrieving this data.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// Server-defined value indicating the response is paginated. Pass this
    /// to the next call to resume where this call left off. Omitted when
    /// there are no additional pages after this one.
    pub marker: Option<Marker<'a>>,
}

/// Represents a single trust line object.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrustLine<'a> {
    /// The unique Address of the counterparty to this trust line.
    pub account: Cow<'a, str>,
    /// Representation of the numeric balance currently held against this line.
    /// A positive balance means that the perspective account holds value; a
    /// negative balance means that the perspective account owes value.
    pub balance: Cow<'a, str>,
    /// A Currency Code identifying what currency this trust line can hold.
    pub currency: Cow<'a, str>,
    /// The maximum amount of the given currency that this account is willing
    /// to owe the peer account.
    pub limit: Cow<'a, str>,
    /// The maximum amount of currency that the counterparty account is willing
    /// to owe the perspective account.
    pub limit_peer: Cow<'a, str>,
    /// Rate at which the account values incoming balances on this trust line,
    /// as a ratio of this value per 1 billion units. (For example, a value of
    /// 500 million represents a 0.5:1 ratio.) As a special case, 0 is treated
    /// as a 1:1 ratio.
    pub quality_in: Option<u32>,
    /// Rate at which the account values outgoing balances on this trust line,
    /// as a ratio of this value per 1 billion units. (For example, a value of
    /// 500 million represents a 0.5:1 ratio.) As a special case, 0 is treated
    /// as a 1:1 ratio.
    pub quality_out: Option<u32>,
    /// (May be omitted) If true, this account has enabled the No Ripple flag
    /// for this trust line. If present and false, this account has disabled
    /// the No Ripple flag, but, because the account also has the Default
    /// Ripple flag disabled, that is not considered the default state. If
    /// omitted, the account has the No Ripple flag disabled for this trust
    /// line and Default Ripple enabled. Updated in: rippled 1.7.0
    pub no_ripple: Option<bool>,
    /// (May be omitted) If true, the peer account has enabled the No Ripple
    /// flag for this trust line. If present and false, this account has
    /// disabled the No Ripple flag, but, because the account also has the
    /// Default Ripple flag disabled, that is not considered the default state.
    /// If omitted, the account has the No Ripple flag disabled for this trust
    /// line and Default Ripple enabled. Updated in: rippled 1.7.0
    pub no_ripple_peer: Option<bool>,
    /// (May be omitted) If true, this account has authorized this trust line.
    /// The default is false.
    pub authorized: Option<bool>,
    /// (May be omitted) If true, the peer account has authorized this trust
    /// line. The default is false.
    pub peer_authorized: Option<bool>,
    /// (May be omitted) If true, this account has frozen this trust line.
    /// The default is false.
    pub freeze: Option<bool>,
    /// (May be omitted) If true, the peer account has frozen this trust line.
    /// The default is false.
    pub freeze_peer: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_lines_deserialization() {
        let json = r#"{
            "account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
            "lines": [
                {
                    "account": "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z",
                    "balance": "0",
                    "currency": "ASP",
                    "limit": "0",
                    "limit_peer": "10",
                    "quality_in": 0,
                    "quality_out": 0
                },
                {
                    "account": "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z",
                    "balance": "0",
                    "currency": "XAU",
                    "limit": "0",
                    "limit_peer": "0",
                    "no_ripple": true,
                    "no_ripple_peer": true,
                    "quality_in": 0,
                    "quality_out": 0
                },
                {
                    "account": "rs9M85karFkCRjvc6KMWn8Coigm9cbcgcx",
                    "balance": "0",
                    "currency": "015841551A748AD2C1F76FF6ECB0CCCD00000000",
                    "limit": "10.01037626125837",
                    "limit_peer": "0",
                    "no_ripple": true,
                    "quality_in": 0,
                    "quality_out": 0
                }
            ],
            "status": "success"
        }"#;

        let account_lines: AccountLines = serde_json::from_str(json).unwrap();

        assert_eq!(account_lines.account, "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59");
        assert_eq!(account_lines.lines.len(), 3);

        // Test first trust line
        let first_line = &account_lines.lines[0];
        assert_eq!(first_line.account, "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z");
        assert_eq!(first_line.balance, "0");
        assert_eq!(first_line.currency, "ASP");
        assert_eq!(first_line.limit, "0");
        assert_eq!(first_line.limit_peer, "10");
        assert_eq!(first_line.quality_in, Some(0));
        assert_eq!(first_line.quality_out, Some(0));
        assert_eq!(first_line.no_ripple, None);
        assert_eq!(first_line.no_ripple_peer, None);

        // Test second trust line
        let second_line = &account_lines.lines[1];
        assert_eq!(second_line.account, "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z");
        assert_eq!(second_line.currency, "XAU");
        assert_eq!(second_line.no_ripple, Some(true));
        assert_eq!(second_line.no_ripple_peer, Some(true));

        // Test third trust line
        let third_line = &account_lines.lines[2];
        assert_eq!(third_line.account, "rs9M85karFkCRjvc6KMWn8Coigm9cbcgcx");
        assert_eq!(
            third_line.currency,
            "015841551A748AD2C1F76FF6ECB0CCCD00000000"
        );
        assert_eq!(third_line.limit, "10.01037626125837");
        assert_eq!(third_line.no_ripple, Some(true));
        assert_eq!(third_line.no_ripple_peer, None);
    }
}
