use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use bigdecimal::BigDecimal;
use core::str::FromStr;

use crate::core::addresscodec::is_valid_classic_address;
use crate::models::amount::XRPAmount;
use crate::models::{
    Amount, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPLModelException, XRPLModelResult,
};

use super::vault_common::validate_vault_id;
use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Claw back assets from a vault holder on the XRP Ledger (XLS-65).
///
/// The issuer of the vault's asset can claw back deposited assets from a
/// specific holder, burning the holder's share tokens in the process.
///
/// See VaultClawback transaction:
/// `<https://github.com/XRPLF/XRPL-Standards/tree/master/XLS-0065d-single-asset-vault>`
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
pub struct VaultClawback<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The ID of the vault to claw back from (256-bit hex string).
    #[serde(rename = "VaultID")]
    pub vault_id: Cow<'a, str>,
    /// The account address of the holder whose assets are being clawed back.
    pub holder: Cow<'a, str>,
    /// The asset amount to claw back. Omit to claw back all funds up to the
    /// total shares the Holder owns.
    pub amount: Option<Amount<'a>>,
}

impl Model for VaultClawback<'_> {
    fn get_errors(&self) -> XRPLModelResult<()> {
        self.validate_currencies()?;
        validate_vault_id(&self.vault_id)?;
        if !is_valid_classic_address(self.holder.as_ref()) {
            return Err(XRPLModelException::InvalidValue {
                field: "holder".into(),
                expected: "a valid classic account address".into(),
                found: self.holder.as_ref().into(),
            });
        }
        if let Some(amount) = &self.amount {
            let value = match amount {
                Amount::MPTAmount(amount) => amount.value.as_ref(),
                Amount::IssuedCurrencyAmount(amount) => amount.value.as_ref(),
                Amount::XRPAmount(amount) => {
                    return Err(XRPLModelException::InvalidValue {
                        field: "amount".into(),
                        expected: "an IOU or MPT amount, or omitted".into(),
                        found: amount.0.to_string(),
                    });
                }
            };
            let parsed = BigDecimal::from_str(value).map_err(|_| {
                XRPLModelException::InvalidValueFormat {
                    field: "amount".into(),
                    format: "a valid decimal number".into(),
                    found: value.into(),
                }
            })?;
            // xrpld VaultClawback preflight: zero amount is valid (means "all").
            // Only negative values are temBAD_AMOUNT.
            if parsed < 0 {
                return Err(XRPLModelException::InvalidValue {
                    field: "amount".into(),
                    expected: "a nonnegative amount".into(),
                    found: value.into(),
                });
            }
        }
        Ok(())
    }
}

impl<'a> Transaction<'a, NoFlags> for VaultClawback<'a> {
    fn get_common_fields(&self) -> &CommonFields<'_, NoFlags> {
        &self.common_fields
    }

    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn get_transaction_type(&self) -> &TransactionType {
        self.common_fields.get_transaction_type()
    }
}

impl<'a> CommonTransactionBuilder<'a, NoFlags> for VaultClawback<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> VaultClawback<'a> {
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
        vault_id: Cow<'a, str>,
        holder: Cow<'a, str>,
        amount: Option<Amount<'a>>,
    ) -> VaultClawback<'a> {
        VaultClawback {
            common_fields: CommonFields::new(
                account,
                TransactionType::VaultClawback,
                account_txn_id,
                fee,
                Some(FlagCollection::default()),
                last_ledger_sequence,
                memos,
                None, // network_id
                sequence,
                signers,
                None, // signing_pub_key
                source_tag,
                ticket_sequence,
                None, // txn_signature
            ),
            vault_id,
            holder,
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::Amount;
    use crate::utils::testing::test_constants::*;

    const VAULT_ID: &str = "A0000000000000000000000000000000000000000000000000000000DEADBEEF";

    #[test]
    fn test_serde() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rIssuer123".into(),
                transaction_type: TransactionType::VaultClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rHolder456".into(),
            amount: Some("500".into()),
        };

        let json_str = r#"{"Account":"rIssuer123","TransactionType":"VaultClawback","Flags":0,"SigningPubKey":"","VaultID":"A0000000000000000000000000000000000000000000000000000000DEADBEEF","Holder":"rHolder456","Amount":"500"}"#;

        // Serialize
        let serialized = serde_json::to_string(&vault_clawback).unwrap();
        assert_eq!(
            serde_json::to_value(&serialized).unwrap(),
            serde_json::to_value(json_str).unwrap()
        );

        // Deserialize
        let deserialized: VaultClawback = serde_json::from_str(json_str).unwrap();
        assert_eq!(vault_clawback, deserialized);
    }

    #[test]
    fn test_serde_no_amount() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rIssuerNoAmt789".into(),
                transaction_type: TransactionType::VaultClawback,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rHolderNoAmt012".into(),
            amount: None,
        };

        let serialized = serde_json::to_string(&vault_clawback).unwrap();
        let deserialized: VaultClawback = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vault_clawback, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rIssuer123".into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rHolder456".into(),
            amount: Some("500".into()),
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_last_ledger_sequence(7108682)
        .with_source_tag(12345)
        .with_memo(Memo {
            memo_data: Some("clawback from holder".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert_eq!(vault_clawback.vault_id, VAULT_ID);
        assert_eq!(vault_clawback.holder, "rHolder456");
        assert_eq!(vault_clawback.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_clawback.common_fields.sequence, Some(100));
        assert_eq!(
            vault_clawback.common_fields.last_ledger_sequence,
            Some(7108682)
        );
        assert_eq!(vault_clawback.common_fields.source_tag, Some(12345));
        assert_eq!(
            vault_clawback.common_fields.memos.as_ref().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_default() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rIssuer789".into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rHolder012".into(),
            amount: Some("100000".into()),
        };

        assert_eq!(vault_clawback.common_fields.account, "rIssuer789");
        assert_eq!(
            vault_clawback.common_fields.transaction_type,
            TransactionType::VaultClawback
        );
        assert_eq!(vault_clawback.vault_id, VAULT_ID);
        assert_eq!(vault_clawback.holder, "rHolder012");
        assert!(vault_clawback.common_fields.fee.is_none());
        assert!(vault_clawback.common_fields.sequence.is_none());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rTicketIssuer111".into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rTicketHolder222".into(),
            amount: Some("2000000".into()),
        }
        .with_ticket_sequence(54321)
        .with_fee("12".into());

        assert_eq!(ticket_clawback.common_fields.ticket_sequence, Some(54321));
        assert!(ticket_clawback.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rMultiMemoIssuer333".into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rMultiMemoHolder444".into(),
            amount: Some("1000".into()),
        }
        .with_memo(Memo {
            memo_data: Some("compliance action".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("regulatory requirement".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_clawback
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_clawback.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_new_constructor() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: "rNewIssuer555".into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                last_ledger_sequence: Some(7108682),
                sequence: Some(100),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "rNewHolder666".into(),
            amount: Some(Amount::IssuedCurrencyAmount(
                crate::models::amount::IssuedCurrencyAmount::new(
                    "XRP".into(),
                    "rNewIssuer555".into(),
                    "750".into(),
                ),
            )),
        };

        assert_eq!(vault_clawback.common_fields.account, "rNewIssuer555");
        assert_eq!(
            vault_clawback.common_fields.transaction_type,
            TransactionType::VaultClawback
        );
        assert_eq!(vault_clawback.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(vault_clawback.vault_id, VAULT_ID);
        assert_eq!(vault_clawback.holder, "rNewHolder666");
    }

    #[test]
    fn test_validate() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(
                crate::models::IssuedCurrencyAmount::new(
                    "USD".into(),
                    ACCOUNT_ISSUER.into(),
                    "100".into(),
                )
                .into(),
            ),
        }
        .with_fee("12".into())
        .with_sequence(300);

        assert!(vault_clawback.validate().is_ok());
    }

    #[test]
    fn test_clawback_all_no_amount() {
        let vault_clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(200),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: None,
        };

        assert!(vault_clawback.amount.is_none());
        assert!(vault_clawback.validate().is_ok());
    }

    #[test]
    fn test_holder_invalid_rejected() {
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: "notanaddress".into(),
            amount: None,
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_amount_xrp_rejected() {
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::XRPAmount("500".into())),
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_amount_zero_accepted() {
        // xrpld VaultClawback: zero amount is valid ("claw back all"). Only negative
        // is temBAD_AMOUNT. Explicit zero is equivalent to omitting the field.
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::IssuedCurrencyAmount(
                crate::models::amount::IssuedCurrencyAmount::new(
                    "USD".into(),
                    ACCOUNT_HOLDER_2.into(),
                    "0".into(),
                ),
            )),
        };
        assert!(clawback.validate().is_ok());
    }

    #[test]
    fn test_amount_negative_rejected() {
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::IssuedCurrencyAmount(
                crate::models::amount::IssuedCurrencyAmount::new(
                    "USD".into(),
                    ACCOUNT_HOLDER_2.into(),
                    "-100".into(),
                ),
            )),
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_amount_not_a_number_rejected() {
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::IssuedCurrencyAmount(
                crate::models::amount::IssuedCurrencyAmount::new(
                    "USD".into(),
                    ACCOUNT_HOLDER_2.into(),
                    "not-a-number".into(),
                ),
            )),
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_amount_valid_ica_accepted() {
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::IssuedCurrencyAmount(
                crate::models::amount::IssuedCurrencyAmount::new(
                    "USD".into(),
                    ACCOUNT_HOLDER_2.into(),
                    "100".into(),
                ),
            )),
        };
        assert!(clawback.validate().is_ok());
    }

    #[test]
    fn test_amount_mpt_positive_accepted() {
        use crate::models::amount::MPTAmount;
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::MPTAmount(MPTAmount {
                mpt_issuance_id: "000000016B4E90A4B36D74F6E16A5BED41EBD7AA37B19B89".into(),
                value: "500".into(),
            })),
        };
        assert!(clawback.validate().is_ok());
    }

    #[test]
    fn test_amount_mpt_negative_rejected() {
        use crate::models::amount::MPTAmount;
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                fee: Some("12".into()),
                sequence: Some(1),
                ..Default::default()
            },
            vault_id: VAULT_ID.into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: Some(Amount::MPTAmount(MPTAmount {
                mpt_issuance_id: "000000016B4E90A4B36D74F6E16A5BED41EBD7AA37B19B89".into(),
                value: "-1".into(),
            })),
        };
        assert!(clawback.validate().is_err());
    }

    #[test]
    fn test_vault_id_invalid_rejected() {
        // vault_id too short
        let clawback = VaultClawback {
            common_fields: CommonFields {
                account: ACCOUNT_ISSUER.into(),
                transaction_type: TransactionType::VaultClawback,
                ..Default::default()
            },
            vault_id: "TOOSHORT".into(),
            holder: ACCOUNT_HOLDER.into(),
            amount: None,
        };
        assert!(clawback.validate().is_err());
    }
}
