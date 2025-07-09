use alloc::borrow::Cow;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    Model, ValidateCurrencies,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

use crate::models::amount::{IssuedCurrencyAmount, XRPAmount};

use super::{CommonFields, CommonTransactionBuilder, FlagCollection};

/// Transactions of the TrustSet type support additional values
/// in the Flags field. This enum represents those options.
///
/// See TrustSet flags:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/trustset>`
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
/// `<https://xrpl.org/docs/references/protocol/transactions/types/trustset>`
#[skip_serializing_none]
#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    xrpl_rust_macros::ValidateCurrencies,
)]
#[serde(rename_all = "PascalCase")]
pub struct TrustSet<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, TrustSetFlag>,
    /// Object defining the trust line to create or modify, in the format of a Currency Amount.
    pub limit_amount: IssuedCurrencyAmount<'a>,
    /// Value incoming balances on this trust line at the ratio of this number per
    /// 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_in: Option<u32>,
    /// Value outgoing balances on this trust line at the ratio of this number per
    /// 1,000,000,000 units. A value of 0 is shorthand for treating balances at face value.
    pub quality_out: Option<u32>,
}

impl<'a> Model for TrustSet<'a> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

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

impl<'a> CommonTransactionBuilder<'a, TrustSetFlag> for TrustSet<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, TrustSetFlag> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
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

    pub fn with_quality_in(mut self, quality_in: u32) -> Self {
        self.quality_in = Some(quality_in);
        self
    }

    pub fn with_quality_out(mut self, quality_out: u32) -> Self {
        self.quality_out = Some(quality_out);
        self
    }

    pub fn with_flag(mut self, flag: TrustSetFlag) -> Self {
        self.common_fields.flags.0.push(flag);
        self
    }

    pub fn with_flags(mut self, flags: Vec<TrustSetFlag>) -> Self {
        self.common_fields.flags = flags.into();
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

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: TrustSet = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let trust_set = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            ..Default::default()
        }
        .with_flag(TrustSetFlag::TfClearNoRipple)
        .with_quality_in(1000000000)
        .with_quality_out(500000000)
        .with_fee("12".into())
        .with_sequence(12)
        .with_last_ledger_sequence(8007750)
        .with_source_tag(12345);

        assert_eq!(trust_set.limit_amount.currency, "USD");
        assert_eq!(
            trust_set.limit_amount.issuer,
            "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc"
        );
        assert_eq!(trust_set.limit_amount.value, "100");
        assert_eq!(trust_set.quality_in, Some(1000000000));
        assert_eq!(trust_set.quality_out, Some(500000000));
        assert!(trust_set.has_flag(&TrustSetFlag::TfClearNoRipple));
        assert_eq!(trust_set.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(trust_set.common_fields.sequence, Some(12));
        assert_eq!(trust_set.common_fields.last_ledger_sequence, Some(8007750));
        assert_eq!(trust_set.common_fields.source_tag, Some(12345));
    }

    #[test]
    fn test_multiple_flags() {
        let trust_set = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            ..Default::default()
        }
        .with_flags(vec![TrustSetFlag::TfSetAuth, TrustSetFlag::TfSetNoRipple])
        .with_fee("12".into());

        assert!(trust_set.has_flag(&TrustSetFlag::TfSetAuth));
        assert!(trust_set.has_flag(&TrustSetFlag::TfSetNoRipple));
        assert!(!trust_set.has_flag(&TrustSetFlag::TfClearNoRipple));
    }

    #[test]
    fn test_default() {
        let trust_set = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "100".into(),
            ),
            ..Default::default()
        };

        assert_eq!(
            trust_set.common_fields.account,
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"
        );
        assert_eq!(
            trust_set.common_fields.transaction_type,
            TransactionType::TrustSet
        );
        assert_eq!(trust_set.limit_amount.currency, "USD");
        assert!(trust_set.quality_in.is_none());
        assert!(trust_set.quality_out.is_none());
    }

    #[test]
    fn test_freeze_operations() {
        let freeze_trust_line = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "0".into(), // Setting to 0 doesn't delete, just modifies flags
            ),
            ..Default::default()
        }
        .with_flag(TrustSetFlag::TfSetFreeze)
        .with_fee("12".into());

        assert!(freeze_trust_line.has_flag(&TrustSetFlag::TfSetFreeze));

        let unfreeze_trust_line = TrustSet {
            common_fields: CommonFields {
                account: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
                transaction_type: TransactionType::TrustSet,
                ..Default::default()
            },
            limit_amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsP3mgGb2tcYUrxiLFiHJiQXhsziegtwBc".into(),
                "0".into(),
            ),
            ..Default::default()
        }
        .with_flag(TrustSetFlag::TfClearFreeze)
        .with_fee("12".into());

        assert!(unfreeze_trust_line.has_flag(&TrustSetFlag::TfClearFreeze));
    }
}
