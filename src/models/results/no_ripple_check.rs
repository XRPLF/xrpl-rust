use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::tx::Transaction;

/// Response from a noripple_check request, which identifies problems with the
/// NoRipple flag settings on an account's trust lines.
///
/// A successful result contains information about potential problems with the
/// account's NoRipple settings and optionally includes transactions to fix
/// these problems.
///
/// See No Ripple Check:
/// `<https://xrpl.org/noripple_check.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NoRippleCheck<'a> {
    /// The ledger index of the ledger used to calculate these results.
    pub ledger_current_index: Option<u32>,
    /// Array of human-readable strings describing the problems. This includes
    /// up to one entry if the account's Default Ripple setting is not as
    /// recommended, plus up to 'limit' entries for trust lines whose No Ripple
    /// setting is not as recommended.
    pub problems: Cow<'a, [Cow<'a, str>]>,
    /// If the request specified transactions as true, this contains an array of
    /// transactions that should fix the described problems. The length of this
    /// array matches the problems array, and each entry corresponds to fixing
    /// the problem at the same index.
    pub transactions: Option<Cow<'a, [Transaction<'a>]>>,
    /// Whether this response contains validated ledger information.
    pub validated: bool,
}

#[cfg(test)]
mod tests {
    use crate::models::{transactions::TransactionType, Amount};

    use super::*;

    #[test]
    fn test_no_ripple_check_deserialization() {
        let json = r#"{
            "ledger_current_index": 14342939,
            "problems": [
                "You should immediately set your default ripple flag",
                "You should clear the no ripple flag on your XAU line to r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z",
                "You should clear the no ripple flag on your USD line to rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q"
            ],
            "transactions": [
                {
                    "Account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
                    "Fee": 10000,
                    "Sequence": 1406,
                    "SetFlag": 8,
                    "TransactionType": "AccountSet"
                },
                {
                    "Account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
                    "Fee": 10000,
                    "Flags": 262144,
                    "LimitAmount": {
                        "currency": "XAU",
                        "issuer": "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z",
                        "value": "0"
                    },
                    "Sequence": 1407,
                    "TransactionType": "TrustSet"
                },
                {
                    "Account": "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59",
                    "Fee": 10000,
                    "Flags": 262144,
                    "LimitAmount": {
                        "currency": "USD",
                        "issuer": "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q",
                        "value": "5"
                    },
                    "Sequence": 1408,
                    "TransactionType": "TrustSet"
                }
            ],
            "validated": false
        }"#;

        let response: NoRippleCheck = serde_json::from_str(json).unwrap();
        let response_clone = response.clone();

        // Test basic fields
        assert_eq!(response.ledger_current_index, Some(14342939));
        assert_eq!(response.validated, false);
        assert_eq!(response.problems.len(), 3);

        // Test problems array
        assert_eq!(
            response.problems[0],
            "You should immediately set your default ripple flag"
        );
        assert_eq!(
            response.problems[1],
            "You should clear the no ripple flag on your XAU line to r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z"
        );
        assert_eq!(
            response.problems[2],
            "You should clear the no ripple flag on your USD line to rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q"
        );

        // Test transactions
        let transactions = response.transactions.unwrap();
        assert_eq!(transactions.len(), 3);

        // Test AccountSet transaction
        match &transactions[0] {
            Transaction::AccountSet {
                account,
                fee,
                sequence,
                set_flag,
                transaction_type,
            } => {
                assert_eq!(account, "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59");
                assert_eq!(*fee, 10000);
                assert_eq!(*sequence, 1406);
                assert_eq!(*set_flag, 8);
                assert_eq!(*transaction_type, TransactionType::AccountSet);
            }
            _ => panic!("Expected AccountSet transaction"),
        }

        // Test first TrustSet transaction
        match &transactions[1] {
            Transaction::TrustSet {
                account,
                fee,
                flags,
                limit_amount,
                sequence,
                transaction_type,
            } => {
                assert_eq!(account, "r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59");
                assert_eq!(*fee, 10000);
                assert_eq!(*flags, 262144);
                match limit_amount {
                    Amount::IssuedCurrencyAmount(issued_amount) => {
                        assert_eq!(issued_amount.currency, "XAU");
                        assert_eq!(issued_amount.issuer, "r3vi7mWxru9rJCxETCyA1CHvzL96eZWx5z");
                        assert_eq!(issued_amount.value, "0");
                    }
                    _ => panic!("Expected IssuedCurrencyAmount"),
                }
                assert_eq!(*sequence, 1407);
                assert_eq!(*transaction_type, TransactionType::TrustSet);
            }
            _ => panic!("Expected TrustSet transaction"),
        }

        // Test serialization
        let serialized = serde_json::to_string(&response_clone).unwrap();
        let deserialized: NoRippleCheck = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response_clone, deserialized);
    }
}
