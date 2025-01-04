use core::str::FromStr;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use bigdecimal::{num_bigint::Sign, BigDecimal};

use crate::{
    models::{
        ledger::objects::LedgerEntryType, transactions::metadata::TransactionMetadata, Amount,
    },
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
        let xrp_value = if value.sign() == Sign::Plus {
            let xrp_quantity = drops_to_xrp(absulte_value.to_string().as_str())?;
            let mut negated_value_string = String::from("-");
            negated_value_string.push_str(&xrp_quantity);

            BigDecimal::from_str(&negated_value_string)?
        } else {
            let xrp_value = drops_to_xrp(absulte_value.to_string().as_str())?;
            let dec = BigDecimal::from_str(&xrp_value)?;
            negate(&dec)
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

fn flip_trustline_perspective(account_balance: AccountBalance) -> XRPLUtilsResult<AccountBalance> {
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
    node: NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance<'a>>> {
    if value.is_none() {
        return Ok(Vec::new());
    }

    let fields = if let Some(new_fields) = node.new_fields {
        new_fields
    } else if let Some(final_fields) = node.final_fields {
        final_fields
    } else {
        return Ok(Vec::new());
    };

    let low_limit = fields.low_limit.as_ref();
    let balance = fields.balance.as_ref();
    let high_limit = fields.high_limit.as_ref();

    if let (Some(low_limit), Some(balance), Some(high_limit)) = (low_limit, balance, high_limit) {
        let low_limit_issuer = low_limit.issuer.as_ref();
        let balance_currency = match balance {
            Amount::IssuedCurrencyAmount(ic) => Some(ic.currency.as_ref()),
            _ => Some("XRP"),
        };
        let high_limit_issuer = high_limit.issuer.as_ref();

        if let Some(balance_currency) = balance_currency {
            let result = AccountBalance {
                account: low_limit_issuer.to_string().into(),
                balance: Balance {
                    currency: balance_currency.to_string().into(),
                    issuer: Some(high_limit_issuer.to_string().into()),
                    value: format!("{}", value.unwrap().normalized()).into(),
                },
            };
            return Ok([result.clone(), flip_trustline_perspective(result)?].into());
        }
    }

    Ok(Vec::new())
}

fn get_node_balances<'a: 'b, 'b>(
    node: NormalizedNode<'a>,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance<'b>>> {
    if node.ledger_entry_type == LedgerEntryType::AccountRoot {
        let xrp_quantity = get_xrp_quantity(node, value)?;
        if let Some(xrp_quantity) = xrp_quantity {
            Ok([xrp_quantity].into())
        } else {
            Ok(Vec::new())
        }
    } else if node.ledger_entry_type == LedgerEntryType::RippleState {
        let trustline_quantities = get_trustline_quantity(node, value)?;
        Ok(trustline_quantities)
    } else {
        Ok(Vec::new())
    }
}

fn group_balances_by_account(account_balances: Vec<AccountBalance>) -> Vec<AccountBalances> {
    let grouped_balances = group_account_balances_by_account(account_balances);
    let mut account_balances_grouped: Vec<AccountBalances> = Vec::new();

    for group in grouped_balances.into_iter() {
        let account = group.account.clone();
        let balances = group
            .account_balances
            .into_iter()
            .map(|balance| balance.balance)
            .collect();
        account_balances_grouped.push(AccountBalances { account, balances });
    }

    account_balances_grouped
}

pub fn derive_account_balances<'a>(
    metadata: &'a TransactionMetadata,
    parser_fn: impl Fn(&NormalizedNode) -> XRPLUtilsResult<Option<BigDecimal>>,
) -> XRPLUtilsResult<Vec<AccountBalances<'a>>> {
    let mut account_balances: Vec<AccountBalance> = Vec::new();
    let nodes = normalize_nodes(metadata);
    for node in nodes.into_iter() {
        let value = parser_fn(&node)?;
        let node_balances = get_node_balances(node, value)?;
        account_balances.extend(node_balances);
    }

    Ok(group_balances_by_account(account_balances))
}
