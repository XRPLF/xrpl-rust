use crate::Err;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::transactions::{CommonFields, XRPLCheckCashException};
use crate::models::NoFlags;
use crate::models::{
    amount::Amount,
    model::Model,
    transactions::{Memo, Signer, Transaction, TransactionType},
};

/// Cancels an unredeemed Check, removing it from the ledger without
/// sending any money. The source or the destination of the check can
/// cancel a Check at any time using this transaction type. If the Check
/// has expired, any address can cancel it.
///
/// See CheckCash:
/// `<https://xrpl.org/checkcash.html>`
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CheckCash<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    // The custom fields for the CheckCash model.
    //
    // See CheckCash fields:
    // `<https://xrpl.org/checkcash.html#checkcash-fields>`
    /// The ID of the Check ledger object to cash, as a 64-character hexadecimal string.
    #[serde(rename = "CheckID")]
    pub check_id: Cow<'a, str>,
    /// Redeem the Check for exactly this amount, if possible. The currency must match that of the
    /// SendMax of the corresponding CheckCreate transaction. You must provide either this field or DeliverMin.
    pub amount: Option<Amount<'a>>,
    /// Redeem the Check for at least this amount and for as much as possible. The currency must
    /// match that of the SendMax of the corresponding CheckCreate transaction. You must provide
    /// either this field or Amount.
    pub deliver_min: Option<Amount<'a>>,
}

impl<'a: 'static> Model for CheckCash<'a> {
    fn get_errors(&self) -> Result<()> {
        match self._get_amount_and_deliver_min_error() {
            Err(error) => Err!(error),
            Ok(_no_error) => Ok(()),
        }
    }
}

impl<'a> Transaction<NoFlags> for CheckCash<'a> {
    fn get_transaction_type(&self) -> TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CheckCashError for CheckCash<'a> {
    fn _get_amount_and_deliver_min_error(&self) -> Result<(), XRPLCheckCashException> {
        if (self.amount.is_none() && self.deliver_min.is_none())
            || (self.amount.is_some() && self.deliver_min.is_some())
        {
            Err(XRPLCheckCashException::DefineExactlyOneOf {
                field1: "amount".into(),
                field2: "deliver_min".into(),
                resource: "".into(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> CheckCash<'a> {
    pub fn new(
        account: Cow<'a, str>,
        account_txn_id: Option<Cow<'a, str>>,
        fee: Option<XRPAmount<'a>>,
        last_ledger_sequence: Option<u32>,
        memos: Option<Vec<Memo>>,
        sequence: Option<u32>,
        signers: Option<Vec<Signer<'a>>>,
        source_tag: Option<u32>,
        ticket_sequence: Option<u32>,
        check_id: Cow<'a, str>,
        amount: Option<Amount<'a>>,
        deliver_min: Option<Amount<'a>>,
    ) -> Self {
        Self {
            common_fields: CommonFields {
                account,
                transaction_type: TransactionType::CheckCash,
                account_txn_id,
                fee,
                flags: None,
                last_ledger_sequence,
                memos,
                sequence,
                signers,
                source_tag,
                ticket_sequence,
            },
            check_id,
            amount,
            deliver_min,
        }
    }
}

pub trait CheckCashError {
    fn _get_amount_and_deliver_min_error(&self) -> Result<(), XRPLCheckCashException>;
}

#[cfg(test)]
mod test_check_cash_error {
    use crate::models::Model;
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_amount_and_deliver_min_error() {
        let check_cash = CheckCash::new(
            "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "".into(),
            None,
            None,
        );

        assert_eq!(
            check_cash.validate().unwrap_err().to_string().as_str(),
            "The field `amount` can not be defined with `deliver_min`. Define exactly one of them. For more information see: "
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let default_txn = CheckCash::new(
            "rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy".into(),
            None,
            Some("12".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".into(),
            Some("100000000".into()),
            None,
        );
        let default_json_str = r#"{"Account":"rfkE1aSy9G8Upk4JssnwBxhEv5p4mn2KTy","TransactionType":"CheckCash","Fee":"12","CheckID":"838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334","Amount":"100000000"}"#;
        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: CheckCash = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }
}
