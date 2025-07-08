use super::utils::{
    balance_parser::derive_account_balances, nodes::NormalizedNode, parser::get_value,
    AccountBalances,
};
use crate::{
    models::transactions::metadata::TransactionMetadata, utils::exceptions::XRPLUtilsResult,
};
use alloc::vec::Vec;
use bigdecimal::BigDecimal;

/// Parses the final balances of all accounts affected by a transaction from the transaction metadata.
pub fn get_final_balances<'a: 'b, 'b>(
    metadata: &'a TransactionMetadata<'a>,
) -> XRPLUtilsResult<Vec<AccountBalances<'b>>> {
    derive_account_balances(metadata, compute_final_balance)
}

fn compute_final_balance(node: &NormalizedNode) -> XRPLUtilsResult<Option<BigDecimal>> {
    let mut value: BigDecimal = BigDecimal::from(0);
    if let Some(new_fields) = &node.new_fields {
        if let Some(balance) = &new_fields.balance {
            value = get_value(&balance.clone().into())?;
        }
    } else if let Some(final_fields) = &node.final_fields {
        if let Some(balance) = &final_fields.balance {
            value = get_value(&balance.clone().into())?;
        }
    }
    if value == BigDecimal::from(0) {
        return Ok(None);
    }
    Ok(Some(value))
}

#[cfg(test)]
mod test {
    use core::cell::LazyCell;

    use serde_json::Value;

    use super::*;
    use crate::{
        models::transactions::metadata::TransactionMetadata, utils::txn_parser::utils::Balance,
    };

    #[test]
    fn test_parse_final_balances() {
        let txn: LazyCell<TransactionMetadata> = LazyCell::new(|| {
            let txn_value: Value =
                serde_json::from_str(include_str!("./test_data/payment_iou.json")).unwrap();
            let txn_meta = txn_value["meta"].clone();
            let txn_meta: TransactionMetadata = serde_json::from_value(txn_meta).unwrap();

            txn_meta
        });
        let expected_balances = Vec::from([
            AccountBalances {
                account: "rKmBGxocj9Abgy25J51Mk1iqFzW9aVF9Tc".into(),
                balances: Vec::from([
                    Balance {
                        currency: "USD".into(),
                        value: "1.525330905250352".into(),
                        issuer: Some("rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q".into()),
                    },
                    Balance {
                        currency: "XRP".into(),
                        value: "-239.555992".into(),
                        issuer: None,
                    },
                ]),
            },
            AccountBalances {
                account: "rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q".into(),
                balances: Vec::from([
                    Balance {
                        currency: "USD".into(),
                        value: "-1.525330905250352".into(),
                        issuer: Some("rKmBGxocj9Abgy25J51Mk1iqFzW9aVF9Tc".into()),
                    },
                    Balance {
                        currency: "USD".into(),
                        value: "-0.02".into(),
                        issuer: Some("rLDYrujdKUfVx28T9vRDAbyJ7G2WVXKo4K".into()),
                    },
                ]),
            },
            AccountBalances {
                account: "rLDYrujdKUfVx28T9vRDAbyJ7G2WVXKo4K".into(),
                balances: Vec::from([Balance {
                    currency: "USD".into(),
                    value: "0.02".into(),
                    issuer: Some("rMwjYedjc7qqtKYVLiAccJSmCwih4LnE2q".into()),
                }]),
            },
        ]);
        let final_balances = get_final_balances(&txn).unwrap();
        assert_eq!(final_balances, expected_balances);
    }
}
