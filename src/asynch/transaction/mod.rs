use super::account::get_next_valid_seq_number;
use super::clients::AsyncClient;
use super::clients::CommonFields;
use super::ledger::get_fee;
use super::ledger::get_latest_validated_ledger_sequence;
use crate::models::amount::XRPAmount;
use crate::models::exceptions::XRPLModelException;
use crate::models::requests::ServerState;
use crate::models::results::ServerState as ServerStateResult;
use crate::models::transactions::Transaction;
use crate::models::transactions::TransactionType;
use crate::models::Model;
use crate::Err;
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use anyhow::Result;
use core::convert::TryInto;
use core::fmt::Debug;
use exceptions::XRPLTransactionException;
use rust_decimal::Decimal;
use serde::Serialize;
use strum::IntoEnumIterator;

pub mod exceptions;

const OWNER_RESERVE: &str = "2000000"; // 2 XRP
const RESTRICTED_NETWORKS: u16 = 1024;
const REQUIRED_NETWORKID_VERSION: &str = "1.11.0";
const LEDGER_OFFSET: u8 = 20;

pub async fn autofill<'a, 'b, F, T>(
    transaction: &mut T,
    client: &'a impl AsyncClient,
    signers_count: Option<u8>,
) -> Result<()>
where
    'a: 'b,
    T: Transaction<'b, F> + Model + Clone,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
{
    let txn = transaction.clone();
    let txn_common_fields = transaction.get_mut_common_fields();
    let common_fields = client.get_common_fields().await?;
    if txn_common_fields.network_id.is_none() && txn_needs_network_id(common_fields.clone())? {
        txn_common_fields.network_id = common_fields.network_id;
    }
    if txn_common_fields.sequence.is_none() {
        txn_common_fields.sequence =
            Some(get_next_valid_seq_number(txn_common_fields.account.clone(), client, None).await?);
    }
    if txn_common_fields.fee.is_none() {
        txn_common_fields.fee =
            Some(calculate_fee_per_transaction_type(txn, Some(client), signers_count).await?);
    }
    if txn_common_fields.last_ledger_sequence.is_none() {
        let ledger_sequence = get_latest_validated_ledger_sequence(client).await?;
        txn_common_fields.last_ledger_sequence = Some(ledger_sequence + LEDGER_OFFSET as u32);
    }

    Ok(())
}

pub async fn calculate_fee_per_transaction_type<'a, 'b, T, F>(
    transaction: T,
    client: Option<&'a impl AsyncClient>,
    signers_count: Option<u8>,
) -> Result<XRPAmount<'_>>
where
    'a: 'b,
    T: Transaction<'b, F>,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
{
    let mut net_fee = XRPAmount::from("10");
    let base_fee;
    if let Some(client) = client {
        net_fee = get_fee(client, None, None).await?;
        base_fee = match transaction.get_transaction_type() {
            TransactionType::EscrowFinish => calculate_base_fee_for_escrow_finish(
                net_fee.clone(),
                transaction
                    .get_field_value("fulfillment")?
                    .map(|fulfillment| fulfillment.into()),
            )?,
            // TODO: same for TransactionType::AMMCreate
            TransactionType::AccountDelete => get_owner_reserve_from_response(client).await?,
            _ => net_fee.clone(),
        };
    } else {
        base_fee = match transaction.get_transaction_type() {
            TransactionType::EscrowFinish => calculate_base_fee_for_escrow_finish(
                net_fee.clone(),
                transaction
                    .get_field_value("fulfillment")?
                    .map(|fulfillment| fulfillment.into()),
            )?,
            // TODO: same for TransactionType::AMMCreate
            TransactionType::AccountDelete => XRPAmount::from(OWNER_RESERVE),
            _ => net_fee.clone(),
        };
    }
    let mut base_fee_decimal: Decimal = base_fee.try_into()?;
    if let Some(signers_count) = signers_count {
        let net_fee_decimal: Decimal = net_fee.try_into()?;
        let signer_count_fee_decimal: Decimal = (1 + signers_count).into();
        base_fee_decimal += &(net_fee_decimal * signer_count_fee_decimal);
    }

    Ok(base_fee_decimal.ceil().into())
}

async fn get_owner_reserve_from_response(client: &impl AsyncClient) -> Result<XRPAmount<'_>> {
    let owner_reserve_response = client.request(ServerState::new(None).into()).await?;
    match owner_reserve_response
        .try_into_result::<ServerStateResult<'_>>()?
        .state
        .validated_ledger
    {
        Some(validated_ledger) => Ok(validated_ledger.reserve_base),
        None => Err!(XRPLModelException::MissingField("validated_ledger")),
    }
}

fn calculate_base_fee_for_escrow_finish<'a>(
    net_fee: XRPAmount<'a>,
    fulfillment: Option<Cow<str>>,
) -> Result<XRPAmount<'a>> {
    if let Some(fulfillment) = fulfillment {
        calculate_based_on_fulfillment(fulfillment, net_fee)
    } else {
        Ok(net_fee)
    }
}

fn calculate_based_on_fulfillment<'a>(
    fulfillment: Cow<str>,
    net_fee: XRPAmount<'_>,
) -> Result<XRPAmount<'a>> {
    let fulfillment_bytes: Vec<u8> = fulfillment.chars().map(|c| c as u8).collect();
    let net_fee_f64: f64 = net_fee.try_into()?;
    let base_fee_string =
        (net_fee_f64 * (33.0 + (fulfillment_bytes.len() as f64 / 16.0))).to_string();
    let base_fee: XRPAmount = base_fee_string.into();
    let base_fee_decimal: Decimal = base_fee.try_into()?;

    Ok(base_fee_decimal.ceil().into())
}

fn txn_needs_network_id(common_fields: CommonFields<'_>) -> Result<bool> {
    let is_higher_restricted_networks = if let Some(network_id) = common_fields.network_id {
        network_id > RESTRICTED_NETWORKS as u32
    } else {
        false
    };
    if let Some(build_version) = common_fields.build_version {
        match is_not_later_rippled_version(REQUIRED_NETWORKID_VERSION.into(), build_version.into())
        {
            Ok(is_not_later_rippled_version) => {
                Ok(is_higher_restricted_networks && is_not_later_rippled_version)
            }
            Err(e) => Err!(e),
        }
    } else {
        Ok(false)
    }
}

fn is_not_later_rippled_version<'a>(
    source: String,
    target: String,
) -> Result<bool, XRPLTransactionException<'a>> {
    if source == target {
        Ok(true)
    } else {
        let source_decomp = source
            .split('.')
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        let target_decomp = target
            .split('.')
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        let (source_major, source_minor) = (
            source_decomp[0]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?,
            source_decomp[1]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?,
        );
        let (target_major, target_minor) = (
            target_decomp[0]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?,
            target_decomp[1]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?,
        );
        if source_major != target_major {
            Ok(source_major < target_major)
        } else if source_minor != target_minor {
            Ok(source_minor < target_minor)
        } else {
            let source_patch = source_decomp[2]
                .split('-')
                .map(|i| i.to_string())
                .collect::<Vec<String>>();
            let target_patch = target_decomp[2]
                .split('-')
                .map(|i| i.to_string())
                .collect::<Vec<String>>();
            let source_patch_version = source_patch[0]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?;
            let target_patch_version = target_patch[0]
                .parse::<u8>()
                .map_err(XRPLTransactionException::ParseRippledVersionError)?;
            if source_patch_version != target_patch_version {
                Ok(source_patch_version < target_patch_version)
            } else if source_patch.len() != target_patch.len() {
                Ok(source_patch.len() < target_patch.len())
            } else if source_patch.len() == 2 {
                if source_patch[1].chars().next().ok_or(
                    XRPLTransactionException::InvalidRippledVersion("source patch version".into()),
                )? != target_patch[1].chars().next().ok_or(
                    XRPLTransactionException::InvalidRippledVersion("target patch version".into()),
                )? {
                    Ok(source_patch[1] < target_patch[1])
                } else if source_patch[1].starts_with('b') {
                    Ok(&source_patch[1][1..] < &target_patch[1][1..])
                } else {
                    Ok(&source_patch[1][2..] < &target_patch[1][2..])
                }
            } else {
                Ok(false)
            }
        }
    }
}

#[cfg(all(feature = "websocket-std", feature = "std", not(feature = "websocket")))]
#[cfg(test)]
mod test_autofill {
    use super::autofill;
    use crate::{
        asynch::clients::{AsyncWebsocketClient, SingleExecutorMutex},
        models::{
            amount::{IssuedCurrencyAmount, XRPAmount},
            transactions::{OfferCreate, Transaction},
        },
    };
    use anyhow::Result;

    #[tokio::test]
    async fn test_autofill_txn() -> Result<()> {
        let mut txn = OfferCreate::new(
            "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            XRPAmount::from("1000000").into(),
            IssuedCurrencyAmount::new(
                "USD".into(),
                "rhub8VRN55s94qWKDv6jmDy1pUykJzF3wq".into(),
                "0.3".into(),
            )
            .into(),
            None,
            None,
        );
        let client = AsyncWebsocketClient::<SingleExecutorMutex, _>::open(
            "wss://testnet.xrpl-labs.com/".parse().unwrap(),
        )
        .await
        .unwrap();
        autofill(&mut txn, &client, None).await?;

        assert!(txn.get_common_fields().network_id.is_none());
        assert!(txn.get_common_fields().sequence.is_some());
        assert!(txn.get_common_fields().fee.is_some());
        assert!(txn.get_common_fields().last_ledger_sequence.is_some());

        Ok(())
    }
}
