pub mod exceptions;
mod multisign;

use core::fmt::Debug;

use crate::{
    asynch::{
        clients::XRPLAsyncClient,
        exceptions::XRPLHelperResult,
        transaction::{
            autofill as async_autofill, autofill_and_sign as async_autofill_and_sign,
            calculate_fee_per_transaction_type as async_calculate_fee_per_transaction_type,
            sign_and_submit as async_sign_and_submit, submit as async_submit,
            submit_and_wait as async_submit_and_wait,
        },
    },
    models::{
        results::{submit::Submit, tx::TxVersionMap},
        transactions::Transaction,
        Model, XRPAmount,
    },
    wallet::Wallet,
};
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
) -> XRPLHelperResult<Submit<'a>>
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
) -> XRPLHelperResult<()>
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
) -> XRPLHelperResult<()>
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

pub fn submit<'a, T, F, C>(transaction: &T, client: &C) -> XRPLHelperResult<Submit<'a>>
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
) -> XRPLHelperResult<TxVersionMap<'b>>
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
) -> XRPLHelperResult<XRPAmount<'c>>
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

// std-only: the no_std AsyncJsonRpcClient is generic over BUF/T/D and can't
// be named without those parameters. The std variant is a simple struct.
#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::asynch::clients::AsyncJsonRpcClient;
    use crate::models::transactions::account_set::AccountSet;

    fn dummy_account_set() -> AccountSet<'static> {
        AccountSet::new(
            "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_calculate_fee_per_transaction_type_no_client() {
        // With client = None, the function returns the default net_fee of 10
        // drops without making any network calls.
        let txn = dummy_account_set();
        let fee: XRPAmount =
            calculate_fee_per_transaction_type::<_, _, AsyncJsonRpcClient>(&txn, None, None)
                .unwrap();
        assert_eq!(fee.0, "10");
    }

    #[test]
    fn test_calculate_fee_per_transaction_type_with_signers_no_client() {
        let txn = dummy_account_set();
        let fee: XRPAmount =
            calculate_fee_per_transaction_type::<_, _, AsyncJsonRpcClient>(&txn, None, Some(3))
                .unwrap();
        // With 3 signers, the fee should be larger than the no-signer baseline.
        let baseline: XRPAmount =
            calculate_fee_per_transaction_type::<_, _, AsyncJsonRpcClient>(&txn, None, None)
                .unwrap();
        let fee_drops: u64 = fee.0.parse().unwrap();
        let baseline_drops: u64 = baseline.0.parse().unwrap();
        assert!(fee_drops >= baseline_drops);
    }
}
