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
