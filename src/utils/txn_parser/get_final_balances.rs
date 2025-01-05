use super::utils::{
    balance_parser::derive_account_balances, nodes::NormalizedNode, parser::get_value,
    AccountBalances,
};
use crate::{
    models::transactions::metadata::TransactionMetadata, utils::exceptions::XRPLUtilsResult,
};
use alloc::vec::Vec;
use bigdecimal::BigDecimal;

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
    use crate::models::transactions::metadata::TransactionMetadata;

    #[test]
    fn test_parse_final_balances() {
        let txn: LazyCell<TransactionMetadata> = LazyCell::new(|| {
            let txn_value: Value =
                serde_json::from_str(include_str!("./test_data/payment_iou.json")).unwrap();
            let txn_meta = txn_value["meta"].clone();
            let txn_meta: TransactionMetadata = serde_json::from_value(txn_meta).unwrap();

            txn_meta
        });
        assert!(get_final_balances(&txn).is_ok());
    }
}
