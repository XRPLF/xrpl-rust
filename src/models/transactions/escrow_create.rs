use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::{Amount, XRPAmount};
use crate::models::transactions::CommonFields;
use crate::models::{
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, ValidateCurrencies,
};
use crate::models::{FlagCollection, NoFlags, XRPLModelException, XRPLModelResult};

use super::CommonTransactionBuilder;

/// Creates an Escrow, which sets aside XRP or a fungible token until the escrow
/// process either finishes or is canceled. Escrowing a token needs the
/// TokenEscrow amendment.
///
/// See EscrowCreate:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/escrowcreate>`
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
pub struct EscrowCreate<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// Amount of XRP, in drops, or of a fungible token (trust-line or MPT), to deduct from the
    /// sender's balance and escrow. Once escrowed, the payment can either go
    /// to the Destination address (after the FinishAfter time) or be
    /// returned to the sender (after the CancelAfter time).
    ///
    /// A token amount needs the TokenEscrow amendment. A trust-line token's
    /// issuer must have the Allow Trust Line Locking flag set
    /// (`tecNO_PERMISSION` otherwise), and an MPT issuance must allow escrow
    /// (`lsfMPTCanEscrow`); the ledger enforces both, not this model.
    pub amount: Amount<'a>,
    /// Address to receive the escrowed amount.
    pub destination: Cow<'a, str>,
    /// Arbitrary tag to further specify the destination for this escrowed
    /// payment, such as a hosted recipient at the destination address.
    pub destination_tag: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, when this
    /// escrow expires. This value is immutable; the funds can
    /// only be returned to the sender after this time.
    pub cancel_after: Option<u32>,
    /// The time, in seconds since the Ripple Epoch, when the escrowed amount
    /// can be released to the recipient. This value is immutable, and the
    /// funds can't be accessed until this time.
    pub finish_after: Option<u32>,
    /// Hex value representing a PREIMAGE-SHA-256 crypto-condition.
    /// The funds can only be delivered to the recipient if this
    /// condition is fulfilled. If the condition is not fulfilled
    /// before the expiration time specified in the CancelAfter
    /// field, the escrowed amount can only revert to the sender.
    pub condition: Option<Cow<'a, str>>,
}

impl<'a> Model for EscrowCreate<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_finish_after_error()?;
        self._get_expiry_error()?;
        self._get_condition_error()?;
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for EscrowCreate<'a> {
    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        self.common_fields.get_mut_common_fields()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for EscrowCreate<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> EscrowCreate<'a> {
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
        amount: impl Into<Amount<'a>>,
        destination: Cow<'a, str>,
        cancel_after: Option<u32>,
        condition: Option<Cow<'a, str>>,
        destination_tag: Option<u32>,
        finish_after: Option<u32>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::EscrowCreate,
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
            amount: amount.into(),
            destination,
            destination_tag,
            cancel_after,
            finish_after,
            condition,
        }
    }

    pub fn with_destination_tag(mut self, destination_tag: u32) -> Self {
        self.destination_tag = Some(destination_tag);
        self
    }

    pub fn with_cancel_after(mut self, cancel_after: u32) -> Self {
        self.cancel_after = Some(cancel_after);
        self
    }

    pub fn with_finish_after(mut self, finish_after: u32) -> Self {
        self.finish_after = Some(finish_after);
        self
    }

    pub fn with_condition(mut self, condition: Cow<'a, str>) -> Self {
        self.condition = Some(condition);
        self
    }
}

impl<'a> EscrowCreate<'a> {
    /// The condition, if any, must be one rippled can read: a DER
    /// PREIMAGE-SHA-256 crypto-condition (see `is_preimage_condition`).
    fn _get_condition_error(&self) -> XRPLModelResult<()> {
        match &self.condition {
            Some(condition) if !is_preimage_condition(condition) => {
                Err(XRPLModelException::InvalidValueFormat {
                    field: "condition".into(),
                    format: "a DER PREIMAGE-SHA-256 crypto-condition, in hex".into(),
                    found: condition.to_string(),
                })
            }
            _ => Ok(()),
        }
    }

    /// rippled's EscrowCreate preflight (3.4.1), for XRP and tokens alike:
    /// an escrow needs FinishAfter or CancelAfter (`temBAD_EXPIRATION`), and
    /// one without FinishAfter needs a Condition (`temMALFORMED`), so it
    /// can't be finished at once.
    fn _get_expiry_error(&self) -> XRPLModelResult<()> {
        if self.finish_after.is_none() && self.cancel_after.is_none() {
            return Err(XRPLModelException::MissingField(
                "finish_after or cancel_after".into(),
            ));
        }
        if self.finish_after.is_none() && self.condition.is_none() {
            return Err(XRPLModelException::FieldRequiresField {
                field1: "cancel_after (without finish_after)".into(),
                field2: "condition".into(),
            });
        }
        Ok(())
    }
}

impl<'a> EscrowCreateError for EscrowCreate<'a> {
    fn _get_finish_after_error(&self) -> XRPLModelResult<()> {
        if let (Some(finish_after), Some(cancel_after)) = (self.finish_after, self.cancel_after) {
            if finish_after >= cancel_after {
                Err(XRPLModelException::ValueBelowValue {
                    field1: "cancel_after".into(),
                    field2: "finish_after".into(),
                    field1_val: cancel_after,
                    field2_val: finish_after,
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

// The DER decoding below (`der_preamble`, `is_preimage_condition` and their
// constants) is a port of rippled's crypto-condition decoder:
// XRPLF/rippled, tag 3.4.1 (commit 00e640751460c137636fdb6dae6b4280de355d81),
// - src/libxrpl/conditions/Condition.cpp (`Condition::deserialize`,
//   `loadSimpleSha256`),
// - include/xrpl/conditions/detail/utils.h (`parsePreamble`, `parseInteger`),
// - include/xrpl/conditions/Condition.h and detail/PreimageSha256.h (limits).
// rippled's notice, which must stay with this code (from its LICENSE.md):
//
//   ISC License
//
//   Copyright (c) 2011, Arthur Britto, David Schwartz, Jed McCaleb, Vinnie
//   Falco, Bob Way, Eric Lombrozo, Nikolaos D. Bougalis, Howard Hinnant.
//   Copyright (c) 2012-present, the XRP Ledger developers.
//
//   Permission to use, copy, modify, and distribute this software for any
//   purpose with or without fee is hereby granted, provided that the above
//   copyright notice and this permission notice appear in all copies.
//
//   THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//   WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//   MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//   ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//   WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//   ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//   OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

/// A DER preamble as rippled's `parsePreamble` reads it: the identifier's class
/// and form bits, its tag, the content length (short or long form), and the
/// bytes after the preamble. `None` where rippled reports an error.
fn der_preamble(s: &[u8]) -> Option<(u8, u8, usize, &[u8])> {
    if s.len() < 2 {
        return None;
    }
    let (kind, tag) = (s[0] & 0xE0, s[0] & 0x1F);
    if tag == 0x1F {
        return None; // long tag form, unsupported
    }
    let mut len = s[1] as usize;
    let mut rest = &s[2..];
    if len & 0x80 != 0 {
        let count = len & 0x7F;
        // rippled's limit is `sizeof(std::size_t)`, and validators are 64-bit,
        // so the count is held to 8 whatever this crate is built for. A
        // `usize` limit would refuse 5- to 8-byte counts on a 32-bit or wasm
        // target that the ledger accepts.
        if count == 0 || count > DER_MAX_LENGTH_BYTES || count > rest.len() {
            return None;
        }
        let long = rest[..count]
            .iter()
            .fold(0u64, |acc, b| (acc << 8) | u64::from(*b));
        rest = &rest[count..];
        if long == 0 {
            return None;
        }
        // A length too large for this target's usize can't fit in the buffer
        // either, so saturating keeps the later "longer than the buffer"
        // refusal, as rippled's `p.length > s.size()` does.
        len = usize::try_from(long).unwrap_or(usize::MAX);
    }
    Some((kind, tag, len, rest))
}

const DER_CONSTRUCTED: u8 = 0x20;
/// The most bytes a long-form DER length may take: rippled's
/// `sizeof(std::size_t)` on the 64-bit builds validators run.
const DER_MAX_LENGTH_BYTES: usize = 8;
const DER_CONTEXT_SPECIFIC: u8 = 0x80;
/// rippled: `Condition::kMaxSerializedCondition` and
/// `PreimageSha256::kMaxPreimageLength`, both 128; the fingerprint is 32 bytes.
const MAX_SERIALIZED_CONDITION: usize = 128;
const MAX_PREIMAGE_LENGTH: u32 = 128;
const FINGERPRINT_SIZE: usize = 32;

/// Whether `condition` (hex, either case) is a crypto-condition rippled 3.4.1
/// accepts for an escrow, following `Condition::deserialize`:
/// - a context-specific, constructed `[0]` (PREIMAGE-SHA-256; the other
///   condition types are unsupported), its contents at most 128 bytes, with
///   nothing after them;
/// - inside, a primitive `[0]` fingerprint of exactly 32 bytes, then a
///   primitive `[1]` cost: an unsigned integer of 1 to 5 bytes (a leading zero
///   octet only at 5), at most 128 (the longest preimage), with nothing after.
pub(crate) fn is_preimage_condition(condition: &str) -> bool {
    let Ok(bytes) = hex::decode(condition) else {
        return false;
    };
    let Some((kind, tag, len, rest)) = der_preamble(&bytes) else {
        return false;
    };
    if kind & DER_CONSTRUCTED == 0 || kind & 0xC0 != DER_CONTEXT_SPECIFIC {
        return false;
    }
    if len > rest.len() || rest.len() > MAX_SERIALIZED_CONDITION || tag != 0 {
        return false;
    }
    if rest.len() != len {
        return false; // trailing garbage after the condition
    }
    // The fingerprint.
    let Some((kind, tag, len, body)) = der_preamble(rest) else {
        return false;
    };
    if kind & DER_CONSTRUCTED != 0 || kind & 0xC0 != DER_CONTEXT_SPECIFIC || tag != 0 {
        return false;
    }
    if len != FINGERPRINT_SIZE || body.len() < len {
        return false;
    }
    // The cost.
    let Some((kind, tag, len, cost)) = der_preamble(&body[FINGERPRINT_SIZE..]) else {
        return false;
    };
    if kind & DER_CONSTRUCTED != 0 || kind & 0xC0 != DER_CONTEXT_SPECIFIC || tag != 1 {
        return false;
    }
    if cost.is_empty() || len > cost.len() || len > 5 || cost[0] & 0x80 != 0 {
        return false;
    }
    if len == 5 && cost[0] != 0 {
        return false;
    }
    if cost.len() != len {
        return false; // a zero-length cost, or bytes after it
    }
    let value = cost.iter().fold(0u64, |acc, b| (acc << 8) | *b as u64);
    value <= MAX_PREIMAGE_LENGTH as u64
}

pub trait EscrowCreateError {
    fn _get_finish_after_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;
    use alloc::format;

    #[test]
    fn test_cancel_after_error() {
        let escrow_create = EscrowCreate {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("100000000").into(),
            destination: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            cancel_after: Some(13298498),
            finish_after: Some(14359039),
            ..Default::default()
        };

        assert!(escrow_create.get_errors().is_err());
    }

    #[test]
    fn test_valid_timing() {
        let escrow_create = EscrowCreate {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("100000000").into(),
            destination: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            cancel_after: Some(14359039),
            finish_after: Some(13298498),
            ..Default::default()
        };

        assert!(escrow_create.get_errors().is_ok());
    }

    #[test]
    fn test_serde() {
        let default_txn = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                source_tag: Some(11747),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: XRPAmount::from("10000").into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            destination_tag: Some(23480),
            cancel_after: Some(533257958),
            finish_after: Some(533171558),
            condition: Some(
                "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"
                    .into(),
            ),
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"EscrowCreate","Flags":0,"SigningPubKey":"","SourceTag":11747,"Amount":"10000","Destination":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","DestinationTag":23480,"CancelAfter":533257958,"FinishAfter":533171558,"Condition":"A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100"}"#;

        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        let deserialized: EscrowCreate = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let escrow_create = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("10000").into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            ..Default::default()
        }
        .with_destination_tag(23480)
        .with_cancel_after(533257958)
        .with_finish_after(533171558)
        .with_condition(
            "A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100".into(),
        )
        .with_source_tag(11747)
        .with_fee("12".into())
        .with_sequence(123);

        assert_eq!(
            escrow_create.amount,
            Amount::XRPAmount(XRPAmount::from("10000"))
        );
        assert_eq!(
            escrow_create.destination,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert_eq!(escrow_create.destination_tag, Some(23480));
        assert_eq!(escrow_create.cancel_after, Some(533257958));
        assert_eq!(escrow_create.finish_after, Some(533171558));
        assert!(escrow_create.condition.is_some());
        assert_eq!(escrow_create.common_fields.source_tag, Some(11747));
        assert_eq!(escrow_create.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(escrow_create.common_fields.sequence, Some(123));
        assert!(escrow_create.get_errors().is_ok());
    }

    #[test]
    fn test_token_escrow_serde() {
        use crate::models::amount::IssuedCurrencyAmount;

        let txn = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "10".into(),
            )
            .into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            cancel_after: Some(533257958),
            finish_after: Some(533171558),
            ..Default::default()
        };
        let json = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"EscrowCreate","Flags":0,"Amount":{"currency":"USD","issuer":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","value":"10"},"Destination":"rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW","CancelAfter":533257958,"FinishAfter":533171558}"#;
        let expected: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_value(&txn).unwrap(), expected);
        let back: EscrowCreate = serde_json::from_str(json).unwrap();
        assert_eq!(back, txn);
        assert!(txn.get_errors().is_ok());
    }

    #[test]
    fn test_expiry_rules_match_rippled() {
        use crate::models::amount::IssuedCurrencyAmount;

        let token = |cancel_after, finish_after, condition: Option<&'static str>| EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: IssuedCurrencyAmount::new(
                "USD".into(),
                "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
                "5".into(),
            )
            .into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            cancel_after,
            finish_after,
            condition: condition.map(Into::into),
            ..Default::default()
        };
        let cond =
            Some("A0258020E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855810100");
        // Neither expiry: refused (temBAD_EXPIRATION on the ledger).
        assert!(token(None, None, cond).get_errors().is_err());
        // CancelAfter only, no Condition: refused (temMALFORMED).
        assert!(token(Some(600), None, None).get_errors().is_err());
        // CancelAfter with a Condition, FinishAfter alone, or both: accepted.
        assert!(token(Some(600), None, cond).get_errors().is_ok());
        assert!(token(None, Some(600), None).get_errors().is_ok());
        assert!(token(Some(700), Some(600), None).get_errors().is_ok());
        // The same rules hold for XRP.
        let mut xrp = token(None, None, cond);
        xrp.amount = XRPAmount::from("1000").into();
        assert!(xrp.get_errors().is_err());
        xrp.finish_after = Some(600);
        assert!(xrp.get_errors().is_ok());
    }

    #[test]
    fn test_mpt_escrow_serde() {
        use crate::models::amount::MPTAmount;

        let id = "00000001A407AF5856CEFBF81F3D4A0000000000A407AF58";
        let txn = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: MPTAmount::new("100".into(), id.into()).into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            finish_after: Some(533171558),
            ..Default::default()
        };
        let value = serde_json::to_value(&txn).unwrap();
        assert_eq!(value["Amount"]["mpt_issuance_id"], id);
        assert_eq!(value["Amount"]["value"], "100");
        let back: EscrowCreate = serde_json::from_value(value).unwrap();
        assert_eq!(back, txn);
        assert!(matches!(back.amount, Amount::MPTAmount(_)));
        assert!(txn.get_errors().is_ok());
        // Through the binary codec and back: the MPT amount survives.
        #[cfg(feature = "core")]
        {
            let blob = crate::core::binarycodec::encode(&txn).unwrap();
            let decoded = crate::core::binarycodec::decode(&blob).unwrap();
            assert_eq!(decoded["Amount"]["mpt_issuance_id"], id);
            assert_eq!(decoded["Amount"]["value"], "100");
            assert_eq!(decoded["TransactionType"], "EscrowCreate");
        }
    }

    #[test]
    fn test_condition_is_checked_as_rippled_reads_it() {
        // SHA-256 of 32 zero bytes, with a 32-byte preimage (cost 0x20).
        let fp = "66687AADF862BD776C8FC18B8E9F8E20089714856EE233B3902A591D0D5F2925";
        let ok = |c: &str| is_preimage_condition(c);
        // Accepted: the usual 32-byte preimage, lower case, cost 0, cost 128
        // (two bytes, as 128 has the high bit), a long-form outer length.
        assert!(ok(&format!("A0258020{fp}810120")));
        assert!(ok(&format!("a0258020{}810120", fp.to_lowercase())));
        assert!(ok(&format!("A0258020{fp}810100")));
        assert!(ok(&format!("A02680 20{fp}81020080").replace(' ', "")));
        assert!(ok(&format!("A0812580 20{fp}810120").replace(' ', "")));
        // Long-form length counts of 5 and 8 bytes, as 64-bit rippled accepts on
        // any target this crate is built for; 9 bytes is refused.
        assert!(ok(
            &format!("A0850000000025 8020{fp}810120").replace(' ', "")
        ));
        assert!(ok(
            &format!("A0880000000000000025 8020{fp}810120").replace(' ', "")
        ));
        assert!(!ok(
            &format!("A089000000000000000025 8020{fp}810120").replace(' ', "")
        ));
        // Refused: empty, not hex, not DER, another condition type.
        for bad in ["", "zz", "00", "A0", "A025"] {
            assert!(!ok(bad), "{bad:?}");
        }
        assert!(
            !ok(&format!("A1258020{fp}810120")),
            "prefix-sha-256 is unsupported"
        );
        // Refused: a 31-byte fingerprint, trailing bytes, a cost over 128,
        // a negative-looking cost, a zero-length cost, a short buffer.
        assert!(!ok(&format!("A024801F{}810120", &fp[..62])));
        assert!(!ok(&format!("A0258020{fp}81012000")));
        assert!(!ok(&format!("A0268020{fp}81020081")));
        assert!(!ok(&format!("A0258020{fp}810181")));
        assert!(!ok(&format!("A0248020{fp}8100")));
        assert!(!ok(&format!("A0308020{fp}810120")));
        // Identifiers: a long tag, a primitive or universal outer identifier.
        assert!(!ok(&format!("BF258020{fp}810120")), "long tag form");
        assert!(!ok(&format!("80258020{fp}810120")), "primitive outer");
        assert!(!ok(&format!("30258020{fp}810120")), "universal outer");
        // Lengths: indefinite, a long form of value 0, a truncated long form.
        assert!(!ok(&format!("A0808020{fp}8101200000")), "indefinite length");
        assert!(!ok("A08100"), "long-form length of 0");
        assert!(!ok("A0840000"), "truncated long-form length");
        // A byte after the cost inside [0], with the outer length counting it.
        assert!(!ok(&format!("A0268020{fp}81012000")), "inner trailing byte");
        // The cost's 5-byte boundary: a leading zero byte then 4 more is read,
        // 6 bytes are not, and the value must still be at most 128.
        assert!(
            ok(&format!("A0298020{fp}81050000000080")),
            "5-byte cost of 128"
        );
        assert!(
            !ok(&format!("A0298020{fp}81050100000000")),
            "5-byte cost over 128"
        );
        assert!(!ok(&format!("A02A8020{fp}8106000000000020")), "6-byte cost");
    }

    #[test]
    fn test_get_errors_refuses_a_bad_condition() {
        let mut txn = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("1000").into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            cancel_after: Some(600),
            condition: Some("".into()),
            ..Default::default()
        };
        assert!(txn.get_errors().is_err());
        txn.condition = Some("DEADBEEF".into());
        assert!(txn.get_errors().is_err());
        txn.condition = Some(
            "A025802066687AADF862BD776C8FC18B8E9F8E20089714856EE233B3902A591D0D5F2925810120".into(),
        );
        assert!(txn.get_errors().is_ok());
    }

    #[test]
    fn test_new_still_takes_xrp_drops() {
        let txn = EscrowCreate::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("10000"),
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            Some(533257958),
            None,
            None,
            None,
        );
        assert_eq!(txn.amount, Amount::XRPAmount(XRPAmount::from("10000")));
    }

    #[test]
    fn test_default() {
        let escrow_create = EscrowCreate {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::EscrowCreate,
                ..Default::default()
            },
            amount: XRPAmount::from("10000").into(),
            destination: "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW".into(),
            ..Default::default()
        };

        assert_eq!(
            escrow_create.common_fields.account,
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"
        );
        assert_eq!(
            escrow_create.common_fields.transaction_type,
            TransactionType::EscrowCreate
        );
        assert_eq!(
            escrow_create.amount,
            Amount::XRPAmount(XRPAmount::from("10000"))
        );
        assert_eq!(
            escrow_create.destination,
            "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW"
        );
        assert!(escrow_create.destination_tag.is_none());
        assert!(escrow_create.cancel_after.is_none());
        assert!(escrow_create.finish_after.is_none());
        assert!(escrow_create.condition.is_none());
    }
}
