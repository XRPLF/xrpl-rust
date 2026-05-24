use alloc::borrow::Cow;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    amount::Amount,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, NoFlags, XRPLModelException, XRPLModelResult,
};

use crate::models::amount::XRPAmount;

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
    /// The amount to be clawed back. Must be an IssuedCurrencyAmount (not XRP).
    /// The account field in the Amount must equal the account field of this transaction.
    pub amount: Amount<'a>,
}

impl<'a> Model for Clawback<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self._get_account_equals_issuer_error()
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

    fn _get_account_equals_issuer_error(&self) -> XRPLModelResult<()> {
        match &self.amount {
            Amount::IssuedCurrencyAmount(currency_amount) => {
                if self.common_fields.account != currency_amount.issuer {
                    Err(XRPLModelException::InvalidValue {
                        field: "amount.issuer".into(),
                        expected: self.common_fields.account.to_string(),
                        found: currency_amount.issuer.to_string(),
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

    #[test]
    fn test_clawback_valid_account_equals_issuer() {
        let account = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        let issuer = account;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                issuer.into(),
                "100".into(),
            ));

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

        assert!(clawback.get_errors().is_ok());
    }

    #[test]
    fn test_clawback_invalid_account_not_equals_issuer() {
        let account = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        let issuer = "rLHzPsX6oXkzU2qL12kHCH8G8cnZv1rBJh";
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                issuer.into(),
                "100".into(),
            ));

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

        assert!(clawback.get_errors().is_err());
        let error = clawback.get_errors().unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("amount.issuer"));
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

        assert!(clawback.get_errors().is_err());
        let error = clawback.get_errors().unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("XRP"));
    }
}
