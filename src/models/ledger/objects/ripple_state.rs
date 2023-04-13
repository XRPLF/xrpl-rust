use crate::_serde::lgr_obj_flags;
use crate::models::ledger::LedgerEntryType;
use crate::models::{amount::Amount, Model};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum_macros::{AsRefStr, Display, EnumIter};

use serde_with::skip_serializing_none;

#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
)]
#[repr(u32)]
pub enum RippleStateFlag {
    /// This RippleState object contributes to the low account's owner reserve.
    LsfLowReserve = 0x00010000,
    /// This RippleState object contributes to the high account's owner reserve.
    LsfHighReserve = 0x00020000,
    /// The low account has authorized the high account to hold tokens issued by the low account.
    LsfLowAuth = 0x00040000,
    /// The high account has authorized the low account to hold tokens issued by the high account.
    LsfHighAuth = 0x00080000,
    /// The low account has disabled rippling from this trust line.
    LsfLowNoRipple = 0x00100000,
    /// The high account has disabled rippling from this trust line.
    LsfHighNoRipple = 0x00200000,
    /// The low account has frozen the trust line, preventing the high account from
    /// transferring the asset.
    LsfLowFreeze = 0x00400000,
    /// The high account has frozen the trust line, preventing the low account from
    /// transferring the asset.
    LsfHighFreeze = 0x00800000,
}

/// The RippleState object type connects two accounts in a single currency. Conceptually,
/// a RippleState object represents two trust lines between the accounts, one from each side.
///
/// `<https://xrpl.org/ripplestate.html#ripplestate>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RippleState<'a> {
    /// The value 0x0072, mapped to the string RippleState, indicates that this object
    /// is a RippleState object.
    ledger_entry_type: LedgerEntryType,
    /// A bit-map of boolean options enabled for this object.
    #[serde(with = "lgr_obj_flags")]
    flags: Vec<RippleStateFlag>,
    /// The object ID of a single object to retrieve from the ledger, as a
    /// 64-character (256-bit) hexadecimal string.
    #[serde(rename = "index")]
    pub index: Cow<'a, str>,
    /// The balance of the trust line, from the perspective of the low account. A negative
    /// balance indicates that the high account holds tokens issued by the low account.
    pub balance: Amount<'a>,
    /// The limit that the high account has set on the trust line. The issuer is the address
    /// of the high account that set this limit.
    pub high_limit: Amount<'a>,
    /// (Omitted in some historical ledgers) A hint indicating which page of the high account's
    /// owner directory links to this object, in case the directory consists of multiple pages.
    pub high_node: Cow<'a, str>,
    /// The limit that the low account has set on the trust line. The issuer is the address of
    /// the low account that set this limit.
    pub low_limit: Amount<'a>,
    /// Omitted in some historical ledgers) A hint indicating which page of the low account's
    /// owner directory links to this object, in case the directory consists of multiple pages.
    pub low_node: Cow<'a, str>,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Cow<'a, str>,
    /// The index of the ledger that contains the transaction that most recently
    /// modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The inbound quality set by the high account, as an integer in the implied ratio
    /// HighQualityIn: 1,000,000,000.
    pub high_quality_in: Option<u32>,
    /// The outbound quality set by the high account, as an integer in the implied ratio
    /// HighQualityOut: 1,000,000,000.
    pub high_quality_out: Option<u32>,
    /// The inbound quality set by the low account, as an integer in the implied ratio
    /// LowQualityIn: 1,000,000,000.
    pub low_quality_in: Option<u32>,
    /// The outbound quality set by the low account, as an integer in the implied ratio
    /// LowQualityOut: 1,000,000,000.
    pub low_quality_out: Option<u32>,
}

impl<'a> Default for RippleState<'a> {
    fn default() -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::RippleState,
            flags: Default::default(),
            index: Default::default(),
            balance: Default::default(),
            high_limit: Default::default(),
            high_node: Default::default(),
            low_limit: Default::default(),
            low_node: Default::default(),
            previous_txn_id: Default::default(),
            previous_txn_lgr_seq: Default::default(),
            high_quality_in: Default::default(),
            high_quality_out: Default::default(),
            low_quality_in: Default::default(),
            low_quality_out: Default::default(),
        }
    }
}

impl<'a> Model for RippleState<'a> {}

impl<'a> RippleState<'a> {
    pub fn new(
        flags: Vec<RippleStateFlag>,
        index: Cow<'a, str>,
        balance: Amount<'a>,
        high_limit: Amount<'a>,
        high_node: Cow<'a, str>,
        low_limit: Amount<'a>,
        low_node: Cow<'a, str>,
        previous_txn_id: Cow<'a, str>,
        previous_txn_lgr_seq: u32,
        high_quality_in: Option<u32>,
        high_quality_out: Option<u32>,
        low_quality_in: Option<u32>,
        low_quality_out: Option<u32>,
    ) -> Self {
        Self {
            ledger_entry_type: LedgerEntryType::RippleState,
            flags,
            index,
            balance,
            high_limit,
            high_node,
            low_limit,
            low_node,
            previous_txn_id,
            previous_txn_lgr_seq,
            high_quality_in,
            high_quality_out,
            low_quality_in,
            low_quality_out,
        }
    }
}

#[cfg(test)]
mod test_serde {
    use super::*;
    use crate::models::amount::IssuedCurrencyAmount;
    use alloc::{borrow::Cow, vec};

    #[test]
    fn test_serialize() {
        let ripple_state = RippleState::new(
            vec![RippleStateFlag::LsfHighReserve, RippleStateFlag::LsfLowAuth],
            Cow::from("9CA88CDEDFF9252B3DE183CE35B038F57282BC9503CDFA1923EF9A95DF0D6F7B"),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rrrrrrrrrrrrrrrrrrrrBZbvji".into(),
                "-10".into(),
            )),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                "110".into(),
            )),
            Cow::from("0000000000000000"),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "0".into(),
            )),
            Cow::from("0000000000000000"),
            Cow::from("E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879"),
            14090896,
            None,
            None,
            None,
            None,
        );
        let ripple_state_json = serde_json::to_string(&ripple_state).unwrap();
        let actual = ripple_state_json.as_str();
        let expected = r#"{"LedgerEntryType":"RippleState","Flags":393216,"index":"9CA88CDEDFF9252B3DE183CE35B038F57282BC9503CDFA1923EF9A95DF0D6F7B","Balance":{"currency":"USD","issuer":"rrrrrrrrrrrrrrrrrrrrBZbvji","value":"-10"},"HighLimit":{"currency":"USD","issuer":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","value":"110"},"HighNode":"0000000000000000","LowLimit":{"currency":"USD","issuer":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","value":"0"},"LowNode":"0000000000000000","PreviousTxnID":"E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879","PreviousTxnLgrSeq":14090896}"#;

        assert_eq!(expected, actual);
    }

    // TODO: test_deserialize
}
