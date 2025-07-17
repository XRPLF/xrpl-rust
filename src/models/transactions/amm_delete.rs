use alloc::{borrow::Cow, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::models::{Currency, FlagCollection, Model, NoFlags, ValidateCurrencies, XRPAmount};

use super::{CommonFields, CommonTransactionBuilder, Memo, Signer, Transaction, TransactionType};

/// Delete an empty Automated Market Maker (AMM) instance that could not be fully
/// deleted automatically.
///
/// Tip: The AMMWithdraw transaction automatically tries to delete an AMM, along with
/// associated ledger entries such as empty trust lines, if it withdrew all the assets
/// from the AMM's pool. However, if there are too many trust lines to the AMM account
/// to remove in one transaction, it may stop before fully removing the AMM. Similarly,
/// an AMMDelete transaction removes up to a maximum number of trust lines; in extreme
/// cases, it may take several AMMDelete transactions to fully delete the trust lines
/// and the associated AMM. In all cases, the AMM ledger entry and AMM account are
/// deleted by the last such transaction.
///
/// See AMMDelete transaction:
/// `<https://xrpl.org/docs/references/protocol/transactions/types/ammdelete>`
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
pub struct AMMDelete<'a> {
    /// The base fields for all transaction models.
    ///
    /// See Transaction Common Fields:
    /// `<https://xrpl.org/transaction-common-fields.html>`
    #[serde(flatten)]
    pub common_fields: CommonFields<'a, NoFlags>,
    /// The definition for one of the assets in the AMM's pool.
    pub asset: Currency<'a>,
    /// The definition for the other asset in the AMM's pool.
    #[serde(rename = "Asset2")]
    pub asset2: Currency<'a>,
}

impl Model for AMMDelete<'_> {
    fn get_errors(&self) -> crate::models::XRPLModelResult<()> {
        self.validate_currencies()
    }
}

impl<'a> Transaction<'a, NoFlags> for AMMDelete<'a> {
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

impl<'a> CommonTransactionBuilder<'a, NoFlags> for AMMDelete<'a> {
    fn get_mut_common_fields(&mut self) -> &mut CommonFields<'a, NoFlags> {
        &mut self.common_fields
    }

    fn into_self(self) -> Self {
        self
    }
}

impl<'a> AMMDelete<'a> {
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
        asset: Currency<'a>,
        asset2: Currency<'a>,
    ) -> AMMDelete<'a> {
        AMMDelete {
            common_fields: CommonFields::new(
                account,
                TransactionType::AMMDelete,
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
            asset,
            asset2,
        }
    }

    // All common builder methods now come from the CommonTransactionBuilder trait!
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IssuedCurrency, currency::XRP};

    #[test]
    fn test_serde() {
        let default_txn = AMMDelete {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMDelete,
                signing_pub_key: Some("".into()),
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
        };

        let default_json_str = r#"{"Account":"rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny","TransactionType":"AMMDelete","Flags":0,"SigningPubKey":"","Asset":{"currency":"XRP"},"Asset2":{"currency":"USD","issuer":"rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd"}}"#;

        // Serialize
        let default_json_value = serde_json::to_value(default_json_str).unwrap();
        let serialized_string = serde_json::to_string(&default_txn).unwrap();
        let serialized_value = serde_json::to_value(&serialized_string).unwrap();
        assert_eq!(serialized_value, default_json_value);

        // Deserialize
        let deserialized: AMMDelete = serde_json::from_str(default_json_str).unwrap();
        assert_eq!(default_txn, deserialized);
    }

    #[test]
    fn test_builder_pattern() {
        let amm_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rJVUeRqDFNs2EQp4ikJUFMdUHURJ8rAqny".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rP9jPyP5kyvFRb6ZiRghAGw5u8SGAmU4bd".into(),
            )),
        }
        .with_fee("12".into()) // From CommonTransactionBuilder trait
        .with_sequence(123) // From CommonTransactionBuilder trait
        .with_last_ledger_sequence(7108682) // From CommonTransactionBuilder trait
        .with_source_tag(12345) // From CommonTransactionBuilder trait
        .with_memo(Memo {
            memo_data: Some("deleting empty AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        }); // From CommonTransactionBuilder trait

        assert_eq!(amm_delete.common_fields.fee.as_ref().unwrap().0, "12");
        assert_eq!(amm_delete.common_fields.sequence, Some(123));
        assert_eq!(amm_delete.common_fields.last_ledger_sequence, Some(7108682));
        assert_eq!(amm_delete.common_fields.source_tag, Some(12345));
        assert_eq!(amm_delete.common_fields.memos.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_default() {
        let amm_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rAMMDeleter123".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "EUR".into(),
                "rEURIssuer456".into(),
            )),
            ..Default::default()
        };

        assert_eq!(amm_delete.common_fields.account, "rAMMDeleter123");
        assert_eq!(
            amm_delete.common_fields.transaction_type,
            TransactionType::AMMDelete
        );
        assert!(matches!(amm_delete.asset, Currency::XRP(_)));
        assert!(matches!(amm_delete.asset2, Currency::IssuedCurrency(_)));
        assert!(amm_delete.common_fields.fee.is_none());
        assert!(amm_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_xrp_token_amm_delete() {
        let xrp_token_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rXRPTokenAMMDeleter789".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "BTC".into(),
                "rBTCIssuer123".into(),
            )),
            ..Default::default()
        }
        .with_fee("12".into())
        .with_sequence(100)
        .with_memo(Memo {
            memo_data: Some("deleting XRP-BTC AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert!(matches!(xrp_token_delete.asset, Currency::XRP(_)));
        assert!(matches!(
            xrp_token_delete.asset2,
            Currency::IssuedCurrency(_)
        ));
        assert_eq!(xrp_token_delete.common_fields.sequence, Some(100));
        assert!(xrp_token_delete.validate().is_ok());
    }

    #[test]
    fn test_token_token_amm_delete() {
        let token_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rTokenAMMDeleter111".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                "rUSDIssuer222".into(),
            )),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "EUR".into(),
                "rEURIssuer333".into(),
            )),
            ..Default::default()
        }
        .with_fee("15".into())
        .with_sequence(200)
        .with_memo(Memo {
            memo_data: Some("cleaning up USD-EUR AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        assert!(matches!(token_delete.asset, Currency::IssuedCurrency(_)));
        assert!(matches!(token_delete.asset2, Currency::IssuedCurrency(_)));
        assert_eq!(token_delete.common_fields.sequence, Some(200));
        assert!(token_delete.validate().is_ok());
    }

    #[test]
    fn test_final_cleanup_delete() {
        let final_cleanup = AMMDelete {
            common_fields: CommonFields {
                account: "rFinalCleanup444".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "DOGE".into(),
                "rDOGEIssuer555".into(),
            )),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "SHIB".into(),
                "rSHIBIssuer666".into(),
            )),
            ..Default::default()
        }
        .with_fee("20".into())
        .with_sequence(300)
        .with_memo(Memo {
            memo_data: Some("final AMM cleanup - removing remaining trust lines".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_source_tag(98765);

        assert_eq!(final_cleanup.common_fields.sequence, Some(300));
        assert_eq!(final_cleanup.common_fields.source_tag, Some(98765));
        assert!(final_cleanup.validate().is_ok());
    }

    #[test]
    fn test_ticket_sequence() {
        let ticket_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rTicketAMMDeleter777".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "ETH".into(),
                "rETHIssuer888".into(),
            )),
            ..Default::default()
        }
        .with_ticket_sequence(54321)
        .with_fee("12".into());

        assert_eq!(ticket_delete.common_fields.ticket_sequence, Some(54321));
        // When using tickets, sequence should be None or 0
        assert!(ticket_delete.common_fields.sequence.is_none());
    }

    #[test]
    fn test_multiple_memos() {
        let multi_memo_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rMultiMemoAMMDeleter999".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::XRP(XRP::new()),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "USDC".into(),
                "rUSDCIssuer111".into(),
            )),
            ..Default::default()
        }
        .with_memo(Memo {
            memo_data: Some("first cleanup attempt".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_memo(Memo {
            memo_data: Some("removing trust lines".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        })
        .with_fee("18".into())
        .with_sequence(400);

        assert_eq!(
            multi_memo_delete
                .common_fields
                .memos
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(multi_memo_delete.common_fields.sequence, Some(400));
    }

    #[test]
    fn test_batch_cleanup_scenario() {
        // Simulate a scenario where multiple AMMDelete transactions are needed
        let batch_deletes: Vec<AMMDelete> = (1..=5)
            .map(|i| {
                AMMDelete {
                    common_fields: CommonFields {
                        account: "rBatchAMMDeleter222".into(),
                        transaction_type: TransactionType::AMMDelete,
                        ..Default::default()
                    },
                    asset: Currency::XRP(XRP::new()),
                    asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                        "USDT".into(),
                        "rUSDTIssuer333".into(),
                    )),
                    ..Default::default()
                }
                .with_fee("12".into())
                .with_sequence(500 + i)
                .with_memo(Memo {
                    memo_data: Some(alloc::format!("cleanup batch {}", i).into()),
                    memo_format: None,
                    memo_type: Some("text".into()),
                })
            })
            .collect();

        assert_eq!(batch_deletes.len(), 5);
        for (i, delete_tx) in batch_deletes.iter().enumerate() {
            assert_eq!(delete_tx.common_fields.sequence, Some(501 + i as u32));
            assert!(delete_tx.validate().is_ok());
        }
    }

    #[test]
    fn test_empty_amm_requirements() {
        // This test documents that AMMDelete should only be used on empty AMMs
        let empty_amm_delete = AMMDelete {
            common_fields: CommonFields {
                account: "rEmptyAMMDeleter444".into(),
                transaction_type: TransactionType::AMMDelete,
                ..Default::default()
            },
            asset: Currency::IssuedCurrency(IssuedCurrency::new(
                "RARE".into(),
                "rRAREIssuer555".into(),
            )),
            asset2: Currency::IssuedCurrency(IssuedCurrency::new(
                "COLLECTOR".into(),
                "rCOLLECTORIssuer666".into(),
            )),
            ..Default::default()
        }
        .with_fee("25".into())
        .with_sequence(600)
        .with_memo(Memo {
            memo_data: Some("deleting empty rare token AMM".into()),
            memo_format: None,
            memo_type: Some("text".into()),
        });

        // The transaction structure is valid even though we can't verify if the AMM is actually empty
        assert!(empty_amm_delete.validate().is_ok());
    }
}
