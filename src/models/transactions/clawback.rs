use alloc::borrow::Cow;
use alloc::format;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::amount::XRPAmount;
use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, NoFlags, XRPLModelException, XRPLModelResult,
};

use super::{CommonFields, CommonTransactionBuilder};

/// Claws back issued currency amount issued by the sender.
///
/// See Clawback:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/clawback>`
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Clawback<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The amount to claw back. For trust line tokens, `Amount.issuer` is the holder.
    pub amount: Amount<'a>,
}

impl<'a> Model for Clawback<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_clawback_amount_error()
    }
}

impl<'a> Transaction<'a, NoFlags> for Clawback<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for Clawback<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> Clawback<'a> {
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
        amount: Amount<'a>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::Clawback,
                account_txn_id,
                fee,
                None,
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
        }
    }

    fn _get_clawback_amount_error(&self) -> XRPLModelResult<()> {
        match &self.amount {
            Amount::IssuedCurrencyAmount(currency_amount) => {
                if self.common_fields.account == currency_amount.issuer {
                    Err(XRPLModelException::ValueEqualsValue {
                        field1: "Account".into(),
                        field2: "Amount.issuer".into(),
                    })
                } else {
                    Ok(())
                }
            }
            Amount::XRPAmount(_) => Err(XRPLModelException::InvalidValue {
                field: "amount".into(),
                expected: "IssuedCurrencyAmount".into(),
                found: "XRP".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::IssuedCurrencyAmount;

    #[test]
    fn adversarial_bug_3_6_account_equals_issuer_should_fail() {
        let issuer = "rsA2LpzuawewSBQXkiju3YQTMzW13pAAdW";
        let clawback = Clawback {
            common_fields: CommonFields {
                account: issuer.into(),
                transaction_type: TransactionType::Clawback,
                ..Default::default()
            },
            amount: Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
                "USD".into(),
                issuer.into(),
                "100".into(),
            )),
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_clawback_valid_issuer_and_holder_differ() {
        let issuer = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        let holder = "rLHzPsX6oXkzU2qL12kHCH8G8cnZv1rBJh";
        let amount = Amount::IssuedCurrencyAmount(IssuedCurrencyAmount::new(
            "USD".into(),
            holder.into(),
            "100".into(),
        ));

        let clawback = Clawback::new(
            issuer.into(),
            None,
            Some("12".into()),
            None,
            None,
            Some(1),
            None,
            None,
            None,
            amount,
        );

        assert!(clawback.validate().is_ok());
    }

    #[test]
    fn test_clawback_rejects_xrp_amount() {
        let account = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        let amount = Amount::XRPAmount("1000000".into());

        let clawback = Clawback::new(
            account.into(),
            None,
            Some("12".into()),
            None,
            None,
            Some(1),
            None,
            None,
            None,
            amount,
        );

        assert!(clawback.validate().is_err());
        let error_msg = format!("{}", clawback.validate().unwrap_err());
        assert!(error_msg.contains("XRP"));
    }
}
