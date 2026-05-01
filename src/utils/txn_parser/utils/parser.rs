use core::str::FromStr;

use alloc::vec::Vec;
use bigdecimal::BigDecimal;

use crate::utils::exceptions::XRPLUtilsResult;

use super::{AccountBalance, AccountObjectGroup, AccountOfferChange, Balance};

pub fn get_value(balance: &Balance) -> XRPLUtilsResult<BigDecimal> {
    Ok(BigDecimal::from_str(balance.value.as_ref())?)
}

pub fn group_balances_by_account(account_balances: Vec<AccountBalance>) -> Vec<AccountObjectGroup> {
    let mut account_object_groups: Vec<AccountObjectGroup> = Vec::new();

    for balance in account_balances.iter() {
        // Find the account object group with the same account. If it doesn't exist, create a new one.
        let account_object_group = account_object_groups
            .iter_mut()
            .find(|group| group.account == balance.account.as_ref());
        if let Some(group) = account_object_group {
            group.account_balances.push(balance.clone());
        } else {
            account_object_groups.push(AccountObjectGroup {
                account: balance.account.clone(),
                account_balances: Vec::new(),
                account_offer_changes: Vec::new(),
            });
            account_object_groups
                .last_mut()
                .unwrap()
                .account_balances
                .push(balance.clone());
        }
    }

    account_object_groups
}

pub fn group_offers_by_account(
    account_offer_changes: Vec<AccountOfferChange>,
) -> Vec<AccountObjectGroup> {
    let mut account_object_groups: Vec<AccountObjectGroup<'_>> = Vec::new();

    for offer_change in account_offer_changes.into_iter() {
        // Find the account object group with the same account. If it doesn't exist, create a new one.
        let account_object_group = account_object_groups
            .iter_mut()
            .find(|group| group.account == offer_change.maker_account.as_ref());
        if let Some(group) = account_object_group {
            group.account_offer_changes.push(offer_change);
        } else {
            account_object_groups.push(AccountObjectGroup {
                account: offer_change.maker_account.clone(),
                account_balances: Vec::new(),
                account_offer_changes: Vec::new(),
            });
            account_object_groups
                .last_mut()
                .unwrap()
                .account_offer_changes
                .push(offer_change);
        }
    }

    account_object_groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transactions::offer_create::OfferCreateFlag;
    use crate::models::XRPAmount;
    use crate::models::{Amount, FlagCollection};
    use crate::utils::txn_parser::utils::{OfferChange, OfferStatus};
    use alloc::vec;

    fn balance(account: &'static str, value: &'static str) -> AccountBalance<'static> {
        AccountBalance {
            account: account.into(),
            balance: Balance {
                currency: "XRP".into(),
                value: value.into(),
                issuer: None,
            },
        }
    }

    fn offer_change(account: &'static str, sequence: u32) -> AccountOfferChange<'static> {
        AccountOfferChange {
            maker_account: account.into(),
            offer_change: OfferChange {
                flags: FlagCollection::<OfferCreateFlag>::default(),
                taker_gets: Amount::XRPAmount(XRPAmount::from("100")),
                taker_pays: Amount::XRPAmount(XRPAmount::from("200")),
                sequence,
                status: OfferStatus::Created,
                maker_exchange_rate: None,
                expiration_time: None,
            },
        }
    }

    #[test]
    fn test_get_value_parses_decimal() {
        let bal = Balance {
            currency: "XRP".into(),
            value: "123.456".into(),
            issuer: None,
        };
        let v = get_value(&bal).unwrap();
        assert_eq!(v.to_string(), "123.456");
    }

    #[test]
    fn test_get_value_invalid_returns_err() {
        let bal = Balance {
            currency: "XRP".into(),
            value: "not-a-number".into(),
            issuer: None,
        };
        assert!(get_value(&bal).is_err());
    }

    #[test]
    fn test_group_balances_by_account_groups_same_account() {
        let balances = vec![
            balance("rA", "1"),
            balance("rB", "2"),
            balance("rA", "3"),
        ];
        let groups = group_balances_by_account(balances);
        assert_eq!(groups.len(), 2);
        let group_a = groups.iter().find(|g| g.account == "rA").unwrap();
        assert_eq!(group_a.account_balances.len(), 2);
        let group_b = groups.iter().find(|g| g.account == "rB").unwrap();
        assert_eq!(group_b.account_balances.len(), 1);
    }

    #[test]
    fn test_group_balances_by_account_empty() {
        let groups = group_balances_by_account(vec![]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_offers_by_account_groups_same_account() {
        let offers = vec![
            offer_change("rA", 1),
            offer_change("rB", 2),
            offer_change("rA", 3),
        ];
        let groups = group_offers_by_account(offers);
        assert_eq!(groups.len(), 2);
        let group_a = groups.iter().find(|g| g.account == "rA").unwrap();
        assert_eq!(group_a.account_offer_changes.len(), 2);
    }

    #[test]
    fn test_group_offers_by_account_empty() {
        let groups = group_offers_by_account(vec![]);
        assert!(groups.is_empty());
    }
}
