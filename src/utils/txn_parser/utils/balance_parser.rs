use core::str::FromStr;

use alloc::{
    string::{String, ToString},
    vec::{self, Vec},
};
use bigdecimal::{num_bigint::Sign, BigDecimal};

use crate::utils::{
    drops_to_xrp,
    exceptions::{XRPLTxnParserException, XRPLUtilsResult},
    txn_parser::utils::{negate, Balance},
};

use super::{nodes::NormalizedNode, AccountBalance};

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

fn flip_trustline_perspective<'a: 'b, 'b>(
    account_balance: &'a AccountBalance<'_>,
) -> XRPLUtilsResult<AccountBalance<'b>> {
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
fn get_trustline_quantity(
    node: NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance>> {
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
    let low_limit = fields.low_limit;
    let balance = fields.balance;
    let high_limit = fields.high_limit;
    if let (Some(low_limit), Some(balance), Some(high_limit)) = (low_limit, balance, high_limit) {
        let low_limit_issuer = low_limit.issuer;
        let balance_currency = balance;
        let high_limit_issuer = high_limit.issuer;
        if let (Some(low_limit_issuer), Some(balance_currency), Some(high_limit_issuer)) =
            (low_limit_issuer, balance_currency, high_limit_issuer)
        {
            let result = AccountBalance {
                account: low_limit_issuer.to_string().into(),
                balance: Balance {
                    currency: balance_currency.to_string().into(),
                    issuer: Some(high_limit_issuer.to_string().into()),
                    value: value.unwrap().normalized().to_string().into(),
                },
            };
            return Ok(vec![result.clone(), flip_trustline_perspective(&result)?]);
        }
    }
    Ok(Vec::new())
}

fn get_node_balances(
    node: NormalizedNode,
    value: Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalance>> {
    match node.ledger_entry_type.as_str() {
        "AccountRoot" => {
            if let Some(xrp_quantity) = get_xrp_quantity(node, value)? {
                Ok(vec![xrp_quantity])
            } else {
                Ok(Vec::new())
            }
        }
        "RippleState" => get_trustline_quantity(node, value),
        _ => Ok(Vec::new()),
    }
}

fn group_balances_by_account(account_balances: Vec<AccountBalance>) -> Vec<AccountBalances> {
    let grouped_balances = group_by_account(account_balances);
    let mut result = Vec::new();
    for (account, account_obj) in grouped_balances {
        let balances: Vec<Balance> = account_obj.into_iter().map(|ab| ab.balance).collect();
        result.push(AccountBalances {
            account: account.to_string().into(),
            balances,
        });
    }
    result
}

fn derive_account_balances(
    metadata: TransactionMetadata,
    parser: impl Fn(NormalizedNode) -> Option<BigDecimal>,
) -> XRPLUtilsResult<Vec<AccountBalances>> {
    let mut quantities = Vec::new();
    for node in normalize_nodes(metadata) {
        if let Some(value) = parser(node.clone()) {
            quantities.extend(get_node_balances(node, Some(value))?);
        }
    }
    Ok(group_balances_by_account(quantities))
}
