use alloc::borrow::Cow;
use anyhow::Result;

use crate::{
    core::addresscodec::{is_valid_xaddress, xaddress_to_classic_address},
    models::{
        ledger::objects::AccountRoot,
        requests::{account_info::AccountInfo, account_tx::AccountTx},
        results, XRPAmount,
    },
    Err,
};

use super::clients::AsyncClient;

pub async fn does_account_exist<C>(
    address: Cow<'_, str>,
    client: &C,
    ledger_index: Option<Cow<'_, str>>,
) -> Result<bool>
where
    C: AsyncClient,
{
    match get_account_root(address, client, ledger_index.unwrap_or("validated".into())).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn get_next_valid_seq_number(
    address: Cow<'_, str>,
    client: &impl AsyncClient,
    ledger_index: Option<Cow<'_, str>>,
) -> Result<u32> {
    let account_info =
        get_account_root(address, client, ledger_index.unwrap_or("current".into())).await?;
    Ok(account_info.sequence)
}

pub async fn get_xrp_balance<'a: 'b, 'b, C>(
    address: Cow<'a, str>,
    client: &C,
    ledger_index: Option<Cow<'a, str>>,
) -> Result<XRPAmount<'b>>
where
    C: AsyncClient,
{
    let account_info =
        get_account_root(address, client, ledger_index.unwrap_or("validated".into())).await?;
    match account_info.balance {
        Some(balance) => Ok(balance),
        None => Ok(0.into()),
    }
}

pub async fn get_account_root<'a: 'b, 'b, C>(
    address: Cow<'a, str>,
    client: &C,
    ledger_index: Cow<'a, str>,
) -> Result<AccountRoot<'b>>
where
    C: AsyncClient,
{
    let mut classic_address = address;
    if is_valid_xaddress(&classic_address) {
        classic_address = match xaddress_to_classic_address(&classic_address) {
            Ok(addr) => addr.0.into(),
            Err(e) => return Err!(e),
        };
    }
    let request = AccountInfo::new(
        None,
        classic_address,
        None,
        Some(ledger_index),
        None,
        None,
        None,
    )
    .into();
    let account_info = client.request(request).await?;

    Ok(account_info
        .try_into_result::<results::account_info::AccountInfo<'_>>()?
        .account_data)
}

pub async fn get_latest_transaction<'a: 'b, 'b, C>(
    mut address: Cow<'a, str>,
    client: &C,
) -> Result<results::account_tx::AccountTx<'b>>
where
    C: AsyncClient,
{
    if is_valid_xaddress(&address) {
        address = match xaddress_to_classic_address(&address) {
            Ok((address, _, _)) => address.into(),
            Err(e) => return Err!(e),
        };
    }
    let account_tx = AccountTx::new(
        None,
        address,
        None,
        Some("validated".into()),
        None,
        None,
        None,
        None,
        Some(1),
        None,
    );
    let response = client.request(account_tx.into()).await?;
    response.try_into_result::<results::account_tx::AccountTx<'_>>()
}
