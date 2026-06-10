use alloc::{borrow::Cow, string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{
    amount::Amount,
    exceptions::XRPLModelException,
    transactions::{Memo, Signer, Transaction, TransactionType},
    Model, NoFlags, ValidateCurrencies, XRPLModelResult,
};

use crate::models::amount::XRPAmount;

use super::{
    mptoken_issuance_set::validate_holder_address, CommonFields, CommonTransactionBuilder,
};

/// Claws back issued currency amount or MPT issued by the sender.
///
/// For IssuedCurrencyAmount: `amount.issuer` must be the token holder's address
/// and `Holder` must be absent.
/// For MPTAmount: `Holder` must be present and must not equal `Account`.
///
/// See Clawback:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/clawback>`
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
pub struct Clawback<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The amount to claw back. Must be IssuedCurrencyAmount or MPTAmount (not XRP).
    /// For ICA: `amount.issuer` must be the holder's address.
    /// For MPT: supply the `holder` field instead.
    pub amount: Amount<'a>,
    /// (MPT only) The account to claw back from. Required when `amount` is
    /// MPTAmount; must not equal the transaction `account`.
    pub holder: Option<Cow<'a, str>>,
}

impl<'a> Model for Clawback<'a> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;
        self._get_account_equals_issuer_error()?;
        self._get_mpt_clawback_error()
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
        holder: Option<Cow<'a, str>>,
    ) -> Self {
        Self {
            common_fields: CommonFields::new(
                account,
                TransactionType::Clawback,
                account_txn_id,
                fee,
                None, // flags: Clawback has no transaction-specific flags.
                last_ledger_sequence,
                memos,
                None, // network_id: let signing/autofill set this when required.
                sequence,
                signers,
                None, // signing_pub_key: populated during signing.
                source_tag,
                ticket_sequence,
                None, // txn_signature: populated during signing.
            ),
            amount,
            holder,
        }
    }

    fn _get_account_equals_issuer_error(&self) -> XRPLModelResult<()> {
        match &self.amount {
            Amount::IssuedCurrencyAmount(currency_amount) => {
                // For ICA clawback: Account is the *issuer* performing the clawback;
                // amount.issuer is the *holder* being clawed from.
                // xrpld rejects self-clawback (Account == amount.issuer).
                if self.common_fields.account == currency_amount.issuer {
                    Err(XRPLModelException::InvalidValue {
                        field: "amount.issuer".into(),
                        expected: "holder address (must differ from Account)".into(),
                        found: currency_amount.issuer.to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            Amount::XRPAmount(_) => Err(XRPLModelException::InvalidValue {
                field: "amount".into(),
                expected: "IssuedCurrencyAmount or MPTAmount".into(),
                found: "XRP".into(),
            }),
            Amount::MPTAmount(_) => Ok(()),
        }
    }

    fn _get_mpt_clawback_error(&self) -> XRPLModelResult<()> {
        match &self.amount {
            Amount::MPTAmount(_) => {
                // MPT clawback: Holder required, must be a valid classic address,
                // and must differ from Account (no self-clawback).
                match &self.holder {
                    None => Err(XRPLModelException::MissingField("holder".into())),
                    Some(holder) if holder == &self.common_fields.account => {
                        Err(XRPLModelException::InvalidValue {
                            field: "holder".into(),
                            expected: "account different from transaction Account".into(),
                            found: holder.to_string(),
                        })
                    }
                    Some(holder) => validate_holder_address(holder.as_ref()),
                }
            }
            Amount::IssuedCurrencyAmount(_) => {
                // ICA clawback: Holder must be absent
                if self.holder.is_some() {
                    Err(XRPLModelException::InvalidValue {
                        field: "holder".into(),
                        expected: "absent for IssuedCurrencyAmount clawback".into(),
                        found: "present".into(),
                    })
                } else {
                    Ok(())
                }
            }
            Amount::XRPAmount(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transactions::test_fixtures::{
        INVALID_MPT_ISSUANCE_ID_NON_HEX, MPT_ISSUANCE_ID, TEST_ACCOUNT, TEST_HOLDER_ACCOUNT,
    };
    use alloc::format;

    fn clawback_tx(
        account: &'static str,
        amount: Amount<'static>,
        holder: Option<&'static str>,
    ) -> Clawback<'static> {
        Clawback {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Clawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            amount,
            holder: holder.map(Into::into),
        }
    }

    #[test]
    fn test_clawback_ica_valid_holder_differs_from_account() {
        // Account = issuer performing clawback; amount.issuer = holder being clawed from.
        // Different addresses = legitimate clawback.
        let account = TEST_ACCOUNT;
        let holder = TEST_HOLDER_ACCOUNT;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                holder.into(),
                "100".into(),
            ));

        let clawback = clawback_tx(account, amount, None);

        assert!(clawback.get_errors().is_ok());
    }

    #[test]
    fn test_clawback_ica_rejects_self_clawback() {
        // Account == amount.issuer → self-clawback; rejected by xrpld.
        let account = TEST_ACCOUNT;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                account.into(),
                "100".into(),
            ));

        let clawback = clawback_tx(account, amount, None);

        assert!(clawback.get_errors().is_err());
        let error_msg = format!("{}", clawback.get_errors().unwrap_err());
        assert!(error_msg.contains("amount.issuer"));
    }

    #[test]
    fn test_clawback_rejects_xrp_amount() {
        let account = TEST_ACCOUNT;
        let amount = Amount::XRPAmount("1000000".into());

        let clawback = clawback_tx(account, amount, None);

        assert!(clawback.get_errors().is_err());
        assert!(clawback._get_mpt_clawback_error().is_ok());
        let error = clawback.get_errors().unwrap_err();
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("XRP"));
    }

    #[test]
    fn test_clawback_mpt_valid() {
        let account = TEST_ACCOUNT;
        let holder = TEST_HOLDER_ACCOUNT;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            MPT_ISSUANCE_ID.into(),
        ));
        let clawback = clawback_tx(account, amount, Some(holder));
        assert!(clawback.get_errors().is_ok());
    }

    #[test]
    fn test_clawback_mpt_rejects_invalid_amount_value() {
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "9223372036854775808".into(),
            MPT_ISSUANCE_ID.into(),
        ));
        let clawback = clawback_tx(TEST_ACCOUNT, amount, Some(TEST_HOLDER_ACCOUNT));

        let error_msg = format!("{}", clawback.get_errors().unwrap_err());
        assert!(error_msg.contains("MPT amount <="));
    }

    #[test]
    fn test_clawback_mpt_rejects_invalid_issuance_id() {
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            INVALID_MPT_ISSUANCE_ID_NON_HEX.into(),
        ));
        let clawback = clawback_tx(TEST_ACCOUNT, amount, Some(TEST_HOLDER_ACCOUNT));

        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_missing_holder() {
        let account = TEST_ACCOUNT;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            MPT_ISSUANCE_ID.into(),
        ));
        let clawback = clawback_tx(account, amount, None);
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_holder_equals_account() {
        let account = TEST_ACCOUNT;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            MPT_ISSUANCE_ID.into(),
        ));
        let clawback = clawback_tx(account, amount, Some(account));
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_mpt_invalid_holder_address() {
        let account = TEST_ACCOUNT;
        let amount = Amount::MPTAmount(crate::models::amount::MPTAmount::new(
            "100".into(),
            MPT_ISSUANCE_ID.into(),
        ));
        let clawback = clawback_tx(account, amount, Some("not-a-valid-xrpl-address"));
        assert!(clawback.get_errors().is_err());
    }

    #[test]
    fn test_clawback_ica_with_holder_rejected() {
        let account = TEST_ACCOUNT;
        let amount =
            Amount::IssuedCurrencyAmount(crate::models::amount::IssuedCurrencyAmount::new(
                "USD".into(),
                TEST_HOLDER_ACCOUNT.into(),
                "100".into(),
            ));
        let clawback = clawback_tx(account, amount, Some(TEST_HOLDER_ACCOUNT));
        assert!(clawback.get_errors().is_err());
        let error_msg = format!("{}", clawback.get_errors().unwrap_err());
        assert!(error_msg.contains("absent for IssuedCurrencyAmount"));
    }
}
