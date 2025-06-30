use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, PathStep, XRPLModelResult,
};

use crate::models::amount::XRPAmount;
use crate::models::transactions::exceptions::XRPLPaymentException;

use super::{CommonFields, FlagCollection};

/// Transactions of the Payment type support additional values
/// in the Flags field. This enum represents those options.
///
/// See Payment flags:
/// `<https://xrpl.org/payment.html#payment-flags>`
#[derive(
    Default,
    Debug,
    Eq,
    PartialEq,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    Display,
    AsRefStr,
    EnumIter,
)]
#[repr(u32)]
pub enum PaymentFlag {
    /// Do not use the default path; only use paths included in the Paths field.
    /// This is intended to force the transaction to take arbitrage opportunities.
    /// Most clients do not need this.
    TfNoDirectRipple = 0x00010000,
    /// If the specified Amount cannot be sent without spending more than SendMax,
    /// reduce the received amount instead of failing outright.
    /// See Partial Payments for more details.
    TfPartialPayment = 0x00020000,
    /// Only take paths where all the conversions have an input:output ratio that
    /// is equal or better than the ratio of Amount:SendMax.
    /// See Limit Quality for details.
    #[default]
    TfLimitQuality = 0x00040000,
}

/// Transfers value from one account to another.
///
/// See Payment:
/// `<https://xrpl.org/payment.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Payment<'a> {
    // The base fields for all transaction models.
    //
    // See Transaction Types:
    // `<https://xrpl.org/transaction-types.html>`
    //
    // See Transaction Common Fields:
    // `<https://xrpl.org/transaction-common-fields.html>`
    /// The type of transaction.
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, PaymentFlag>,
    // The custom fields for the Payment model.
    //
    // See Payment fields:
    // `<https://xrpl.org/payment.html#payment-fields>`
    /// The amount of currency to deliver. For non-XRP amounts, the nested field names
    /// MUST be lower-case. If the tfPartialPayment flag is set, deliver up to this
    /// amount instead.
    pub amount: Amount<'a>,
    /// The unique address of the account receiving the payment.
    pub destination: Cow<'a, str>,
    /// Arbitrary tag that identifies the reason for the payment to the destination,
    /// or a hosted recipient to pay.
    pub destination_tag: Option<u32>,
    /// Arbitrary 256-bit hash representing a specific reason or identifier for this payment.
    pub invoice_id: Option<u32>,
    /// Array of payment paths to be used for this transaction. Must be omitted for
    /// XRP-to-XRP transactions.
    pub paths: Option<Vec<Vec<PathStep<'a>>>>,
    /// Highest amount of source currency this transaction is allowed to cost, including
    /// transfer fees, exchange rates, and slippage . Does not include the XRP destroyed
    /// as a cost for submitting the transaction. For non-XRP amounts, the nested field
    /// names MUST be lower-case. Must be supplied for cross-currency/cross-issue payments.
    /// Must be omitted for XRP-to-XRP payments.
    pub send_max: Option<Amount<'a>>,
    /// Minimum amount of destination currency this transaction should deliver. Only valid
    /// if this is a partial payment. For non-XRP amounts, the nested field names are lower-case.
    pub deliver_min: Option<Amount<'a>>,
}

impl<'a: 'static> Model for Payment<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_xrp_transaction_error()?;
        self._get_partial_payment_error()?;
        self._get_exchange_error()?;

        Ok(())
    }
}

impl<'a> Transaction<'a, PaymentFlag> for Payment<'a> {
    fn has_flag(&self, flag: &PaymentFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }

    fn get_common_fields(&self) -> &CommonFields<'_, PaymentFlag> {
        self.common_fields.get_common_fields()
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, PaymentFlag> {
        &mut self.common_fields
    }
}

impl<'a> PaymentError for Payment<'a> {
    fn _get_xrp_transaction_error(&self) -> XRPLModelResult<()> {
        if self.amount.is_xrp() && self.send_max.is_none() {
            if self.paths.is_some() {
                Err(XRPLPaymentException::IllegalOption {
                    field: "paths".into(),
                    context: "XRP to XRP payments".into(),
                }
                .into())
            } else if self.common_fields.account == self.destination {
                Err(XRPLPaymentException::ValueEqualsValueInContext {
                    field1: "account".into(),
                    field2: "destination".into(),
                    context: "XRP to XRP Payments".into(),
                }
                .into())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_partial_payment_error(&self) -> XRPLModelResult<()> {
        if let Some(send_max) = &self.send_max {
            if !self.has_flag(&PaymentFlag::TfPartialPayment)
                && send_max.is_xrp()
                && self.amount.is_xrp()
            {
                Err(XRPLPaymentException::IllegalOption {
                    field: "send_max".into(),
                    context: "XRP to XRP non-partial payments".into(),
                }
                .into())
            } else {
                Ok(())
            }
        } else if self.has_flag(&PaymentFlag::TfPartialPayment) {
            Err(XRPLPaymentException::FlagRequiresField {
                flag: PaymentFlag::TfPartialPayment,
                field: "send_max".into(),
            }
            .into())
        } else if !self.has_flag(&PaymentFlag::TfPartialPayment) {
            if let Some(_deliver_min) = &self.deliver_min {
                Err(XRPLPaymentException::IllegalOption {
                    field: "deliver_min".into(),
                    context: "XRP to XRP non-partial payments".into(),
                }
                .into())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_exchange_error(&self) -> XRPLModelResult<()> {
        if self.common_fields.account == self.destination && self.send_max.is_none() {
            return Err(XRPLPaymentException::OptionRequired {
                field: "send_max".into(),
                context: "exchanges".into(),
            }
            .into());
        }

        Ok(())
    }
}

impl<'a> Payment<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        flags: Option<FlagCollection<PaymentFlag>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        amount: Amount<'a>,
        destination: Cow<'a, str>,
        deliver_min: Option<Amount<'a>>,
        destination_tag: Option<u32>,
        invoice_id: Option<u32>,
        paths: Option<Vec<Vec<PathStep<'a>>>>,
        send_max: Option<Amount<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::Payment,
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
            amount,
            destination,
            destination_tag,
            invoice_id,
            paths,
            send_max,
            deliver_min,
        }
    }

    /// Set destination tag
    pub fn with_destination_tag(mut self, tag: u32) -> Self {
        self.destination_tag = Some(tag);
        self
    }

    /// Set send max
    pub fn with_send_max(mut self, send_max: Amount<'a>) -> Self {
        self.send_max = Some(send_max);
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
    pub fn with_flag(mut self, flag: PaymentFlag) -> Self {
        // flags is not an Option, it's directly a FlagCollection
        self.common_fields.flags.0.push(flag); // Access the inner Vec directly
        self
    }

    /// Set multiple flags at once
    pub fn with_flags(mut self, flags: Vec<PaymentFlag>) -> Self {
        self.common_fields.flags = flags.into();
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
}

impl<'a> Default for Payment<'a> {
    fn default() -> Self {
        Self {
            common_fields: CommonFields::default(),
            amount: Amount::XRPAmount("0".into()),
            destination: "".into(),
            destination_tag: None,
            invoice_id: None,
            paths: None,
            send_max: None,
            deliver_min: None,
        }
    }
}

pub trait PaymentError {
    fn _get_xrp_transaction_error(&self) -> XRPLModelResult<()>;
    fn _get_partial_payment_error(&self) -> XRPLModelResult<()>;
    fn _get_exchange_error(&self) -> XRPLModelResult<()>;
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::models::amount::{Amount, IssuedCurrencyAmount, XRPAmount};
    use crate::models::{Model, PathStep};

    use super::*;

    #[test]
    fn test_xrp_to_xrp_error() {
        let mut payment = Payment {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            paths: Some(vec![vec![
                PathStep::default().with_account("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into())
            ]]),
            ..Default::default()
        };

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `\"paths\"` is not allowed to be defined for \"XRP to XRP payments\""
        );

        payment.paths = None;
        payment.send_max = Some(Amount::XRPAmount(XRPAmount::from("99999")));

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `\"send_max\"` is not allowed to be defined for \"XRP to XRP non-partial payments\""
        );

        payment.send_max = None;
        payment.destination = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into();

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The value of the field `\"account\"` is not allowed to be the same as the value of the field `\"destination\"`, for \"XRP to XRP Payments\""
        );
    }

    #[test]
    fn test_partial_payments_error() {
        let payment = Payment {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::Payment,
                flags: vec![PaymentFlag::TfPartialPayment].into(),
                ..Default::default()
            },
            amount: Amount::XRPAmount("1000000".into()),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            ..Default::default()
        };

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "For the flag `TfPartialPayment` to be set it is required to define the field `\"send_max\"`"
        );

        let payment = Payment {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::XRPAmount("1000000".into()),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            deliver_min: Some(Amount::XRPAmount("99999".into())),
            ..Default::default()
        };

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `\"deliver_min\"` is not allowed to be defined for \"XRP to XRP non-partial payments\""
        );
    }

    #[test]
    fn test_exchange_error() {
        let payment = Payment {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                "10".into(),
            )),
            destination: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            ..Default::default()
        };

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `\"send_max\"` is required to be defined for \"exchanges\""
        );
    }

    #[test]
    fn test_serde() {
        let default_txn = Payment {
            common_fields: CommonFields {
                account: "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                transaction_type: TransactionType::Payment,
                fee: Some("12".into()),
                flags: vec![PaymentFlag::TfPartialPayment].into(),
                sequence: Some(2),
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                "1".into(),
            )),
            destination: "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            ..Default::default()
        };

        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"Payment","Fee":"12","Flags":131072,"Sequence":2,"SigningPubKey":"","Amount":{"currency":"USD","issuer":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","value":"1"},"Destination":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: Payment = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}

#[cfg(all(feature = "helpers", feature = "wallet"))]
#[cfg(test)]
mod test_sign {
    use alloc::vec;

    use crate::{
        asynch::{exceptions::XRPLHelperResult, transaction::sign},
        models::transactions::Transaction,
        wallet::Wallet,
    };

    use super::*;

    #[test]
    fn test_payment_sign_with_memo() -> XRPLHelperResult<()> {
        let mut payment = Payment {
            common_fields: CommonFields {
                account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
                transaction_type: TransactionType::Payment,
                memos: Some(vec![Memo {
                    memo_data: Some("68656c6c6f".into()),
                    memo_format: None,
                    memo_type: Some("74657874".into()),
                }]),
                ..Default::default()
            },
            amount: Amount::XRPAmount("1000000".into()),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            ..Default::default()
        };

        let wallet = Wallet::create(None)?;
        sign(&mut payment, &wallet, false)?;

        assert!(payment.get_common_fields().is_signed());

        Ok(())
    }
}
