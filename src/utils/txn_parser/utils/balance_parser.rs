use core::str::FromStr;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bigdecimal::{num_bigint::Sign, BigDecimal};

use crate::{
    models::transactions::metadata::TransactionMetadata,
    utils::{
        drops_to_xrp,
        exceptions::{XRPLTxnParserException, XRPLUtilsResult},
        txn_parser::utils::{negate, nodes::normalize_nodes, Balance},
    },
};

use super::{
    nodes::NormalizedNode, parser::group_balances_by_account as group_account_balances_by_account,
    AccountBalance, AccountBalances,
};

fn get_xrp_quantity(
    node: NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Option<AccountBalance>> {
    if let Some(value) = value {
        let absulte_value = value.clone().abs();
        let xrp_value = if value.sign() != Sign::Minus {
            let xrp_quantity = drops_to_xrp(absulte_value.to_string().as_str())?;
            let mut negated_value_string = String::from("-");
            negated_value_string.push_str(&xrp_quantity);

            BigDecimal::from_str(&negated_value_string)?
        } else {
            let xrp_value = drops_to_xrp(absulte_value.to_string().as_str())?;

            BigDecimal::from_str(&xrp_value)?
        };
        if let Some(final_fields) = node.final_fields {
            if let Some(account) = final_fields.account {
                Ok(Some(AccountBalance {
                    account: account.to_string().into(),
                    balance: Balance {
                        currency: "XRP".into(),
                        value: xrp_value.to_string().into(),
                        issuer: None,
                    },
                }))
            } else {
                Ok(None)
            }
        } else if let Some(new_fields) = node.new_fields {
            if let Some(account) = new_fields.account {
                Ok(Some(AccountBalance {
                    account: account.to_string().into(),
                    balance: Balance {
                        currency: "XRP".into(),
                        value: xrp_value.to_string().into(),
                        issuer: None,
                    },
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn flip_trustline_perspective<'a>(
    account_balance: &'a AccountBalance,
) -> XRPLUtilsResult<AccountBalance<'a>> {
    let balance = account_balance.balance.clone();
    let negated_value = negate(&BigDecimal::from_str(balance.value.as_ref())?);
    let issuer = balance.issuer.clone();

    Ok(AccountBalance {
        account: issuer.ok_or(XRPLTxnParserException::MissingField("issuer"))?,
        balance: Balance {
            currency: balance.currency,
            value: negated_value.normalized().to_string().into(),
            issuer: Some(account_balance.account.clone()),
        },
    })
}
fn get_trustline_quantity<'a>(
    node: &NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance<'a>>> {
    todo!()
}

fn get_node_balances<'a>(
    node: &NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance<'a>>> {
    todo!()
}

fn group_balances_by_account(account_balances: Vec<AccountBalance>) -> Vec<AccountBalances> {
    let grouped_balances = group_account_balances_by_account(account_balances.as_ref());
    let mut account_balances_grouped: Vec<AccountBalances> = Vec::new();
}

pub fn derive_account_balances<'a>(
    metadata: &TransactionMetadata,
    parser_fn: impl Fn(&NormalizedNode) -> XRPLUtilsResult<Option<BigDecimal>>,
) -> XRPLUtilsResult<Vec<AccountBalances<'a>>> {
    let mut account_balances: Vec<AccountBalance> = Vec::new();
    let nodes = normalize_nodes(metadata);
    for node in nodes.into_iter() {
        let value = parser_fn(&node)?;
        let node_balances = get_node_balances(&node, value)?;
        account_balances.extend(node_balances);
    }

    Ok(group_balances_by_account(account_balances))
}
