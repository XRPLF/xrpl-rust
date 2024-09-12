pub mod exceptions;
mod multisign;

use core::fmt::Debug;

use crate::{
    asynch::{
        clients::XRPLAsyncClient,
        transaction::{
            autofill as async_autofill, autofill_and_sign as async_autofill_and_sign,
            calculate_fee_per_transaction_type as async_calculate_fee_per_transaction_type,
            sign_and_submit as async_sign_and_submit, submit as async_submit,
            submit_and_wait as async_submit_and_wait,
        },
    },
    models::{
        results::{submit::Submit, tx::Tx},
        transactions::Transaction,
        Model, XRPAmount,
    },
    wallet::Wallet,
};
use anyhow::Result;
use embassy_futures::block_on;
use serde::{de::DeserializeOwned, Serialize};
use strum::IntoEnumIterator;

pub use crate::asynch::transaction::sign;
pub use multisign::*;

pub fn sign_and_submit<'a, 'b, T, F, C>(
    transaction: &mut T,
    client: &'b C,
    wallet: &Wallet,
    autofill: bool,
    check_fee: bool,
) -> Result<Submit<'a>>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
    C: XRPLAsyncClient,
{
    block_on(async_sign_and_submit(
        transaction,
        client,
        wallet,
        autofill,
        check_fee,
    ))
}

pub fn autofill<'a, 'b, F, T, C>(
    transaction: &mut T,
    client: &'b C,
    signers_count: Option<u8>,
) -> Result<()>
where
    T: Transaction<'a, F> + Model + Clone,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    C: XRPLAsyncClient,
{
    block_on(async_autofill(transaction, client, signers_count))
}

pub fn autofill_and_sign<'a, 'b, T, F, C>(
    transaction: &mut T,
    client: &'b C,
    wallet: &Wallet,
    check_fee: bool,
) -> Result<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
    C: XRPLAsyncClient,
{
    block_on(async_autofill_and_sign(
        transaction,
        client,
        wallet,
        check_fee,
    ))
}

pub fn submit<'a, T, F, C>(transaction: &T, client: &C) -> Result<Submit<'a>>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
    C: XRPLAsyncClient,
{
    block_on(async_submit(transaction, client))
}

pub fn submit_and_wait<'a: 'b, 'b, T, F, C>(
    transaction: &'b mut T,
    client: &C,
    wallet: Option<&Wallet>,
    check_fee: Option<bool>,
    autofill: Option<bool>,
) -> Result<Tx<'b>>
where
    T: Transaction<'a, F> + Model + Clone + DeserializeOwned + Debug,
    F: IntoEnumIterator + Serialize + Debug + PartialEq + Debug + Clone + 'a,
    C: XRPLAsyncClient,
{
    block_on(async_submit_and_wait(
        transaction,
        client,
        wallet,
        check_fee,
        autofill,
    ))
}

pub fn calculate_fee_per_transaction_type<'a, 'b, 'c, T, F, C>(
    transaction: &T,
    client: Option<&'b C>,
    signers_count: Option<u8>,
) -> Result<XRPAmount<'c>>
where
    T: Transaction<'a, F>,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    C: XRPLAsyncClient,
{
    block_on(async_calculate_fee_per_transaction_type(
        transaction,
        client,
        signers_count,
    ))
}
