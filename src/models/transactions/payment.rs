use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::models::{
    amount::Amount,
    model::Model,
    transactions::{Flag, Memo, Signer, Transaction, TransactionType},
    PathStep,
};

use crate::Err;
use crate::_serde::txn_flags;
use crate::models::amount::XRPAmount;
use crate::models::transactions::XRPLPaymentException;

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
    #[serde(default = "TransactionType::payment")]
    pub transaction_type: TransactionType,
    /// The unique address of the account that initiated the transaction.
    pub account: Cow<'a, str>,
    /// Integer amount of XRP, in drops, to be destroyed as a cost
    /// for distributing this transaction to the network. Some
    /// transaction types have different minimum requirements.
    /// See Transaction Cost for details.
    pub fee: Option<XRPAmount<'a>>,
    /// The sequence number of the account sending the transaction.
    /// A transaction is only valid if the Sequence number is exactly
    /// 1 greater than the previous transaction from the same account.
    /// The special case 0 means the transaction is using a Ticket instead.
    pub sequence: Option<u32>,
    /// Highest ledger index this transaction can appear in.
    /// Specifying this field places a strict upper limit on how long
    /// the transaction can wait to be validated or rejected.
    /// See Reliable Transaction Submission for more details.
    pub last_ledger_sequence: Option<u32>,
    /// Hash value identifying another transaction. If provided, this
    /// transaction is only valid if the sending account's
    /// previously-sent transaction matches the provided hash.
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Cow<'a, str>>,
    /// Hex representation of the public key that corresponds to the
    /// private key used to sign this transaction. If an empty string,
    /// indicates a multi-signature is present in the Signers field instead.
    pub signing_pub_key: Option<Cow<'a, str>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction
    /// is made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub source_tag: Option<u32>,
    /// The sequence number of the ticket to use in place
    /// of a Sequence number. If this is provided, Sequence must
    /// be 0. Cannot be used with AccountTxnID.
    pub ticket_sequence: Option<u32>,
    /// The signature that verifies this transaction as originating
    /// from the account it says it is from.
    pub txn_signature: Option<Cow<'a, str>>,
    /// Set of bit-flags for this transaction.
    #[serde(default)]
    #[serde(with = "txn_flags")]
    pub flags: Option<Vec<PaymentFlag>>,
    /// Additional arbitrary information used to identify this transaction.
    pub memos: Option<Vec<Memo>>,
    /// Arbitrary integer used to identify the reason for this
    /// payment, or a sender on whose behalf this transaction is
    /// made. Conventionally, a refund should specify the initial
    /// payment's SourceTag as the refund payment's DestinationTag.
    pub signers: Option<Vec<Signer<'a>>>,
    /// The custom fields for the Payment model.
    ///
    /// See Payment fields:
    /// `<https://xrpl.org/payment.html#payment-fields>`
    pub amount: Amount<'a>,
    pub destination: Cow<'a, str>,
    pub destination_tag: Option<u32>,
    pub invoice_id: Option<u32>,
    pub paths: Option<Vec<Vec<PathStep<'a>>>>,
    pub send_max: Option<Amount<'a>>,
    pub deliver_min: Option<Amount<'a>>,
}

impl<'a> Default for Payment<'a> {
    fn default() -> Self {
        Self {
            transaction_type: TransactionType::Payment,
            account: Default::default(),
            fee: Default::default(),
            sequence: Default::default(),
            last_ledger_sequence: Default::default(),
            account_txn_id: Default::default(),
            signing_pub_key: Default::default(),
            source_tag: Default::default(),
            ticket_sequence: Default::default(),
            txn_signature: Default::default(),
            flags: Default::default(),
            memos: Default::default(),
            signers: Default::default(),
            amount: Default::default(),
            destination: Default::default(),
            destination_tag: Default::default(),
            invoice_id: Default::default(),
            paths: Default::default(),
            send_max: Default::default(),
            deliver_min: Default::default(),
        }
    }
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

impl<'a> Transaction for Payment<'a> {
    fn has_flag(&self, flag: &Flag) -> bool {
        let mut flags = &Vec::new();

        if let Some(flag_set) = self.flags.as_ref() {
            flags = flag_set;
        }

        match flag {
            Flag::Payment(payment_flag) => match payment_flag {
                PaymentFlag::TfLimitQuality => flags.contains(&PaymentFlag::TfLimitQuality),
                PaymentFlag::TfNoDirectRipple => flags.contains(&PaymentFlag::TfNoDirectRipple),
                PaymentFlag::TfPartialPayment => flags.contains(&PaymentFlag::TfPartialPayment),
            },
            _ => false,
        }
    }

    fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
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
            } else if self.account == self.destination {
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
            if !self.has_flag(&Flag::Payment(PaymentFlag::TfPartialPayment))
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
        } else if self.has_flag(&Flag::Payment(PaymentFlag::TfPartialPayment)) {
            Err(XRPLPaymentException::FlagRequiresField {
                flag: PaymentFlag::TfPartialPayment,
                field: "send_max".into(),
                resource: "".into(),
            })
        } else if !self.has_flag(&Flag::Payment(PaymentFlag::TfPartialPayment)) {
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
        if self.account == self.destination && self.send_max.is_none() {
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
        amount: Amount<'a>,
        destination: Cow<'a, str>,
        fee: Option<XRPAmount<'a>>,
        sequence: Option<u32>,
        last_ledger_sequence: Option<u32>,
        account_txn_id: Option<Cow<'a, str>>,
        signing_pub_key: Option<Cow<'a, str>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        txn_signature: Option<Cow<'a, str>>,
        flags: Option<Vec<PaymentFlag>>,
        memos: Option<Vec<Memo>>,
        signers: Option<Vec<Signer<'a>>>,
        destination_tag: Option<u32>,
        invoice_id: Option<u32>,
        paths: Option<Vec<Vec<PathStep<'a>>>>,
        send_max: Option<Amount<'a>>,
        deliver_min: Option<Amount<'a>>,
    ) -> Self {
        Self {
            transaction_type: TransactionType::Payment,
            account,
            fee,
            sequence,
            last_ledger_sequence,
            account_txn_id,
            signing_pub_key,
            source_tag,
            ticket_sequence,
            txn_signature,
            flags,
            memos,
            signers,
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
        let mut payment = Payment {
            transaction_type: TransactionType::Payment,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            amount: Amount::XRPAmount(XRPAmount::from("1000000")),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            destination_tag: None,
            invoice_id: None,
            paths: Some(vec![vec![PathStep {
                account: Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into()),
                currency: None,
                issuer: None,
                r#type: None,
                type_hex: None,
            }]]),
            send_max: None,
            deliver_min: None,
        };

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
        let mut payment = Payment {
            transaction_type: TransactionType::Payment,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            amount: Amount::XRPAmount("1000000".into()),
            destination: "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK".into(),
            destination_tag: None,
            invoice_id: None,
            paths: None,
            send_max: None,
            deliver_min: None,
        };
        payment.flags = Some(vec![PaymentFlag::TfPartialPayment]);

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "For the flag `TfPartialPayment` to be set it is required to define the field `send_max`. For more information see: "
        );

        payment.flags = None;
        payment.deliver_min = Some(Amount::XRPAmount("99999".into()));

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `deliver_min` is not allowed to be defined for XRP to XRP non-partial payments.For more information see: "
        );
    }

    #[test]
    fn test_exchange_error() {
        let payment = Payment {
            transaction_type: TransactionType::Payment,
            account: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            account_txn_id: None,
            signing_pub_key: None,
            source_tag: None,
            ticket_sequence: None,
            txn_signature: None,
            flags: None,
            memos: None,
            signers: None,
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B".into(),
                "10".into(),
            )),
            destination: "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            destination_tag: None,
            invoice_id: None,
            paths: None,
            send_max: None,
            deliver_min: None,
        };

        assert_eq!(
            payment.validate().unwrap_err().to_string().as_str(),
            "The optional field `send_max` is required to be defined for exchanges. For more information see: "
        );
    }
}

#[cfg(test)]
mod test_serde {
    use alloc::vec;

    use crate::models::amount::{Amount, IssuedCurrencyAmount};

    use super::*;

    #[test]
    fn test_serialize() {
        let default_txn = Payment::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                "1".into(),
            )),
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            Some("12".into()),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![PaymentFlag::TfPartialPayment]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"Payment","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Fee":"12","Sequence":2,"Flags":131072,"Amount":{"currency":"USD","issuer":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","value":"1"},"Destination":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX"}"#;

        let txn_as_string = serde_json::to_string(&default_txn).unwrap();
        let txn_json = txn_as_string.as_str();

        assert_eq!(txn_json, default_json);
    }

    #[test]
    fn test_deserialize() {
        let default_txn = Payment::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
                "1".into(),
            )),
            "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX".into(),
            Some("12".into()),
            Some(2),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![PaymentFlag::TfPartialPayment]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let default_json = r#"{"TransactionType":"Payment","Account":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn","Destination":"ra5nK24KXen9AHvsdFTKHSANinZseWnPcX","Amount":{"currency":"USD","value":"1","issuer":"rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn"},"Fee":"12","Flags":131072,"Sequence":2}"#;

        let txn_as_obj: Payment = serde_json::from_str(default_json).unwrap();

        assert_eq!(txn_as_obj, default_txn);
    }
}
