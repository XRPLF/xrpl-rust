use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model,
};

use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};

use super::{CommonFields, FlagCollection};

/// Transactions of the TrustSet type support additional values
/// in the Flags field. This enum represents those options.
///
/// See TrustSet flags:
/// `<https://xrpl.org/trustset.html#trustset-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum TrustSetFlag {
    /// Authorize the other party to hold currency issued by this account.
    /// (No effect unless using the asfRequireAuth AccountSet flag.) Cannot be unset.
    TfSetAuth = 0x00010000,
    /// Enable the No Ripple flag, which blocks rippling between two trust lines
    /// of the same currency if this flag is enabled on both.
    TfSetNoRipple = 0x00020000,
    /// Disable the No Ripple flag, allowing rippling on this trust line.
    TfClearNoRipple = 0x00040000,
    /// Freeze the trust line.
    TfSetFreeze = 0x00100000,
    /// Unfreeze the trust line.
    TfClearFreeze = 0x00200000,
}

/// Create or modify a trust line linking two accounts.
///
/// See TrustSet:
/// `<https://xrpl.org/trustset.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TrustSet<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, TrustSetFlag>,
    // The custom fields for the TrustSet model.
    //
    // See TrustSet fields:
    // `<https://xrpl.org/trustset.html#trustset-fields>`
    /// Object defining the trust line to create or modify, in the format of a Currency Amount.
    pub limit_amount: IssuedCurrencyAmount<'a>,
    /// Value incoming balances on this trust line at the ratio of this number per
    /// 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_in: Option<u32>,
    /// Value outgoing balances on this trust line at the ratio of this number per
    /// 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_out: Option<u32>,
}

impl<'a> Model for TrustSet<'a> {}

impl<'a> Transaction<'a, TrustSetFlag> for TrustSet<'a> {
    fn has_flag(&self, flag: &TrustSetFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, TrustSetFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, TrustSetFlag> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> Default for TrustSet<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields {
                account: "".into(),
                transaction_type: TransactionType::TrustSet,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::default(),
            quality_in: None,
            quality_out: None,
        }
    }
}

impl<'a> TrustSet<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<TrustSetFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        limit_amount: IssuedCurrencyAmount<'a>,
        quality_in: Option<u32>,
        quality_out: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::TrustSet,
                account_txn_id,
                fee,
                Some(flags.unwrap_or_default()),
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
            limit_amount,
            quality_in,
            quality_out,
        }
    }

    /// Set quality in
    pub fn with_quality_in(mut self, quality_in: u32) -> Self {
        self.quality_in = Some(quality_in);
        self
    }

    /// Set quality out
    pub fn with_quality_out(mut self, quality_out: u32) -> Self {
        self.quality_out = Some(quality_out);
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

    /// Add flag
    pub fn with_flag(mut self, flag: TrustSetFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    /// Set multiple flags
    pub fn with_flags(mut self, flags: Vec<TrustSetFlag>) -> Self {
        self.common_fields.flags = flags.into();
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
    use alloc::vec;

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                fee: Some("12".into()),
                flags: vec![TrustSetFlag::TfClearNoRipple].into(),
                last_ledger_sequence: Some(8007750),
                sequence: Some(12),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            quality_in: None,
            quality_out: None,
        };

        let default_json_str = r#"{"Account":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","TransactionType":"TrustSet","Fee":"12","Flags":262144,"LastLedgerSequence":8007750,"Sequence":12,"SigningPubKey":"","LimitAmount":{"currency":"USD","issuer":"rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc","value":"100"}}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: TrustSet = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
