use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, PathStep,
};

use crate::models::amount::XRPAmount;
use crate::models::transactions::exceptions::XRPLPaymentException;
use crate::Err;

use super::{CommonFields, FlagCollection};

/// Transactions of the Payment type support additional values
/// in the Flags field. This enum represents those options.
///
/// See Payment flags:
/// `<https://xrpl.org/payment.html#payment-flags>`
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize_repr, Deserialize_repr, Display, AsRefStr, EnumIter,
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
    fn get_errors(&self) -> Result<()> {
        match self._get_xrp_transaction_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => match self._get_partial_payment_error() {
                Err(error) => Err!(error),
                Ok(_no_error) => match self._get_exchange_error() {
                    Err(error) => Err!(error),
                    Ok(_no_error) => Ok(()),
                },
            },
        }
    }
}

impl<'a> Transaction<'a, PaymentFlag> for Payment<'a> {
    fn has_flag(&self, flag: &PaymentFlag) -> bool {
        self.common_fields.has_flag(flag)
    }

    fn get_transaction_type(&self) -> TransactionType {
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
    fn _get_xrp_transaction_error(&self) -> Result<(), XRPLPaymentException> {
        if self.amount.is_xrp() && self.send_max.is_none() {
            if self.paths.is_some() {
                Err(XRPLPaymentException::IllegalOption {
                    field: "paths".into(),
                    context: "XRP to XRP payments".into(),
                    resource: "".into(),
                })
            } else if self.common_fields.account == self.destination {
                Err(XRPLPaymentException::ValueEqualsValueInContext {
                    field1: "account".into(),
                    field2: "destination".into(),
                    context: "XRP to XRP Payments".into(),
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_partial_payment_error(&self) -> Result<(), XRPLPaymentException> {
        if let Some(send_max) = &self.send_max {
            if !self.has_flag(&PaymentFlag::TfPartialPayment)
                && send_max.is_xrp()
                && self.amount.is_xrp()
            {
                Err(XRPLPaymentException::IllegalOption {
                    field: "send_max".into(),
                    context: "XRP to XRP non-partial payments".into(),
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else if self.has_flag(&PaymentFlag::TfPartialPayment) {
            Err(XRPLPaymentException::FlagRequiresField {
                flag: PaymentFlag::TfPartialPayment,
                field: "send_max".into(),
                resource: "".into(),
            })
        } else if !self.has_flag(&PaymentFlag::TfPartialPayment) {
            if let Some(_deliver_min) = &self.deliver_min {
                Err(XRPLPaymentException::IllegalOption {
                    field: "deliver_min".into(),
                    context: "XRP to XRP non-partial payments".into(),
                    resource: "".into(),
                })
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn _get_exchange_error(&self) -> Result<(), XRPLPaymentException> {
        if self.common_fields.account == self.destination && self.send_max.is_none() {
            return Err(XRPLPaymentException::OptionRequired {
                field: "send_max".into(),
                context: "exchanges".into(),
                resource: "".into(),
            });
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
        signers: Option<Vec<Signer<'a>>>,
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
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::Payment,
                account_txn_id,
                fee,
                flags: flags.unwrap_or_default(),
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
                network_id: None,
                signing_pub_key: None,
                txn_signature: None,
            },
            amount,
            destination,
            destination_tag,
            invoice_id,
            paths,
            send_max,
            deliver_min,
        }
    }
}

pub trait PaymentError {
    fn _get_xrp_transaction_error(&self) -> Result<(), XRPLPaymentException>;
    fn _get_partial_payment_error(&self) -> Result<(), XRPLPaymentException>;
    fn _get_exchange_error(&self) -> Result<(), XRPLPaymentException>;
}

#[cfg(test)]
mod test_payment_error {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::models::{
        amount::{Amount, IssuedCurrencyAmount, XRPAmount},
        Model, PathStep,
    };

    use super::*;

    #[test]
    fn test_xrp_to_xrp_error() {
        let mut payment = Payment::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount(XRPAmount::from("1000000")),
            "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            None,
            None,
            None,
            Some(vec![vec![PathStep {
                account: Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into()),
                currency: None,
                issuer: None,
                r#type: None,
                type_hex: None,
            }]]),
            None,
        );

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `paths` is not allowed to be defined for XRP to XRP payments.For more information see: "
        );

        payment.paths = None;
        payment.send_max = Some(Amount::XRPAmount(XRPAmount::from("99999")));

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `send_max` is not allowed to be defined for XRP to XRP non-partial payments.For more information see: "
        );

        payment.send_max = None;
        payment.destination = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into();

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The value of the field `account` is not allowed to be the same as the value of the field `destination`, for XRP to XRP Payments. For more information see: "
        );
    }

    #[test]
    fn test_partial_payments_eror() {
        let mut payment = Payment::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::XRPAmount("1000000".into()),
            "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            None,
            None,
            None,
            None,
            None,
        );
        payment.common_fields.flags = vec![PaymentFlag::TfPartialPayment].into();

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "For the flag `TfPartialPayment` to be set it is required to define the field `send_max`. For more information see: "
        );

        payment.common_fields.flags = FlagCollection::default();
        payment.deliver_min = Some(Amount::XRPAmount("99999".into()));

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `deliver_min` is not allowed to be defined for XRP to XRP non-partial payments.For more information see: "
        );
    }

    #[test]
    fn test_exchange_error() {
        let payment = Payment::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                "10".into(),
            )),
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `send_max` is required to be defined for exchanges. For more information see: "
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::models::amount::{Amount, IssuedCurrencyAmount};

    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = Payment::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            Some("12".into()),
            Some(vec![PaymentFlag::TfPartialPayment].into()),
            None,
            None,
            Some(2),
            None,
            None,
            None,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                "1".into(),
            )),
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            None,
            None,
            None,
            None,
            None,
        );
        let default_json_str = r#"{"Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","TransactionType":"Payment","Fee":"12","Flags":131072,"Sequence":2,"Amount":{"currency":"USD","issuer":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","value":"1"},"Destination":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"}"#;
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
