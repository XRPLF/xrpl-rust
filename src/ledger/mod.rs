use embassy_futures::block_on;

use crate::{
    asynch::{
        clients::XRPLClient,
        exceptions::XRPLHelperResult,
        ledger::{
            get_fee as async_get_fee,
            get_latest_open_ledger_sequence as async_get_latest_open_ledger_sequence,
            get_latest_validated_ledger_sequence as async_get_latest_validated_ledger_sequence,
        },
    },
    models::XRPAmount,
};

pub use crate::asynch::ledger::FeeType;

pub fn get_latest_validated_ledger_sequence<C>(client: &C) -> XRPLHelperResult<u32>
where
    C: XRPLClient,
{
    block_on(async_get_latest_validated_ledger_sequence(client))
}

pub fn get_latest_open_ledger_sequence<C>(client: &C) -> XRPLHelperResult<u32>
where
    C: XRPLClient,
{
    block_on(async_get_latest_open_ledger_sequence(client))
}

pub fn get_fee<C>(
    client: &C,
    max_fee: Option<u32>,
    fee_type: Option<FeeType>,
) -> XRPLHelperResult<XRPAmount<'_>>
where
    C: XRPLClient,
{
    block_on(async_get_fee(client, max_fee, fee_type))
}
