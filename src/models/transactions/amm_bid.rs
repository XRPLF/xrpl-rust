use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    transactions::TransactionType, Currency, FlagCollection, IssuedCurrencyAmount, Model, NoFlags,
    XRPAmount,
};

use super::{AuthAccount, CommonFields, Memo, Signer, Transaction};

/// Bid on an Automated Market Maker's (AMM's) auction slot.
///
/// See AMM Bid:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammbid>`
///
/// If you win, you can trade against the AMM at a discounted fee until you are outbid
/// or 24 hours have passed. If you are outbid before 24 hours have passed, you are
/// refunded part of the cost of your bid based on how much time remains. You bid using
/// the AMM's LP Tokens; the amount of a winning bid is returned to the AMM, decreasing
/// the outstanding balance of LP Tokens.
///
/// See AMMBid transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammbid>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AMMBid<'a> {
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
    /// Pay at least this amount of LPTokens for the slot. Setting this value higher
    /// makes it harder for others to outbid you. If omitted, pay the minimum necessary
    /// to win the bid.
    pub bid_min: Option<IssuedCurrencyAmount<'a>>,
    /// Pay at most this amount of LPTokens for the slot. If the cost to win the bid
    /// is higher than this amount, the transaction fails. If omitted, pay as much as
    /// necessary to win the bid.
    pub bid_max: Option<IssuedCurrencyAmount<'a>>,
    /// A list of up to 4 additional accounts that you allow to trade at the discounted
    /// fee. This cannot include the address of the transaction sender.
    pub auth_accounts: Option<Vec<AuthAccount>>,
}
impl Model for AMMBid<'_> {}

impl<'a> Transaction<'a, NoFlags> for AMMBid<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> Default for AMMBid<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::AMMBid,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::default(),
            asset2: Currency::default(),
            bid_min: None,
            bid_max: None,
            auth_accounts: None,
        }
    }
}

impl<'a> AMMBid<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        asset: Currency<'a>,
        asset2: Currency<'a>,
        bid_min: Option<IssuedCurrencyAmount<'a>>,
        bid_max: Option<IssuedCurrencyAmount<'a>>,
        auth_accounts: Option<Vec<AuthAccount>>,
    ) -> AMMBid<'a> {
        AMMBid {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMBid,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None,
                sequence,
                signers,
                None,
                source_tag,
                ticket_sequence,
                None,
            ),
            asset,
            asset2,
            bid_min,
            bid_max,
            auth_accounts,
        }
    }

    /// Set bid minimum
    pub fn with_bid_min(mut self, bid_min: IssuedCurrencyAmount<'a>) -> Self {
        self.bid_min = Some(bid_min);
        self
    }

    /// Set bid maximum
    pub fn with_bid_max(mut self, bid_max: IssuedCurrencyAmount<'a>) -> Self {
        self.bid_max = Some(bid_max);
        self
    }

    /// Set authorized accounts
    pub fn with_auth_accounts(mut self, auth_accounts: Vec<AuthAccount>) -> Self {
        self.auth_accounts = Some(auth_accounts);
        self
    }

    /// Add authorized account
    pub fn add_auth_account(mut self, auth_account: AuthAccount) -> Self {
        if let Some(ref mut accounts) = self.auth_accounts {
            accounts.push(auth_account);
        } else {
            self.auth_accounts = Some(vec![auth_account]);
        }
        self
    }

    /// Set fee
    pub fn with_fee(mut self, fee: XRPAmount<'a>) -> Self {
        self.common_fields.fee = Some(fee);
        self
    }

    /// Set sequence
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.common_fields.sequence = Some(sequence);
        self
    }

    /// Set last ledger sequence
    pub fn with_last_ledger_sequence(mut self, last_ledger_sequence: u32) -> Self {
        self.common_fields.last_ledger_sequence = Some(last_ledger_sequence);
        self
    }

    /// Add memo
    pub fn with_memo(mut self, memo: Memo) -> Self {
        if let Some(ref mut memos) = self.common_fields.memos {
            memos.push(memo);
        } else {
            self.common_fields.memos = Some(vec![memo]);
        }
        self
    }

    /// Set source tag
    pub fn with_source_tag(mut self, source_tag: u32) -> Self {
        self.common_fields.source_tag = Some(source_tag);
        self
    }

    /// Set ticket sequence
    pub fn with_ticket_sequence(mut self, ticket_sequence: u32) -> Self {
        self.common_fields.ticket_sequence = Some(ticket_sequence);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{currency::XRP, IssuedCurrency};

    #[test]
    fn test_serde() {
        let default_txn = AMMBid {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMBid,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            bid_min: Some(IssuedCurrencyAmount::new(
                "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
                "100".into(),
            )),
            bid_max: Some(IssuedCurrencyAmount::new(
                "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
                "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
                "110".into(),
            )),
            auth_accounts: Some(vec![
                AuthAccount {
                    account: "rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg".into(),
                },
                AuthAccount {
                    account: "rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv".into(),
                },
            ]),
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMBid","Flags":0,"SigningPubKey":"","Asset":{"currency":"XRP"},"Asset2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"},"BidMin":{"currency":"039C99CD9AB0B70B32ECDA51EAAE471625608EA2","issuer":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","value":"100"},"BidMax":{"currency":"039C99CD9AB0B70B32ECDA51EAAE471625608EA2","issuer":"rE54zDvgnghAoPopCgvtiqWNq3dU5y836S","value":"110"},"AuthAccounts":[{"AuthAccount":{"Account":"rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg"}},{"AuthAccount":{"Account":"rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv"}}]}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AMMBid = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let bid = AMMBid {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMBid,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
            ..Default::default()
        }
        .with_bid_min(IssuedCurrencyAmount::new(
            "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
            "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
            "100".into(),
        ))
        .with_bid_max(IssuedCurrencyAmount::new(
            "039C99CD9AB0B70B32ECDA51EAAE471625608EA2".into(),
            "rE54zDvgnghAoPopCgvtiqWNq3dU5y836S".into(),
            "110".into(),
        ))
        .add_auth_account(AuthAccount {
            account: "rMKXGCbJ5d8LbrqthdG46q3f969MVK2Qeg".into(),
        })
        .add_auth_account(AuthAccount {
            account: "rBepJuTLFJt3WmtLXYAxSjtBWAeQxVbncv".into(),
        })
        .with_fee("12".into())
        .with_sequence(123);

        assert_eq!(bid.bid_min.as_ref().unwrap().value, Cow::from("100"));
        assert_eq!(bid.bid_max.as_ref().unwrap().value, Cow::from("110"));
        assert_eq!(bid.auth_accounts.as_ref().unwrap().len(), 2);
        assert_eq!(bid.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(bid.common_fields.sequence, Some(123));
    }
}
