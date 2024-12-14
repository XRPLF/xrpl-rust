pub mod exceptions;
mod submit_and_wait;

use bigdecimal::{BigDecimal, RoundingMode};
pub use submit_and_wait::*;

use crate::{
    asynch::{
        account::get_next_valid_seq_number,
        clients::{CommonFields, XRPLAsyncClient},
        ledger::{get_fee, get_latest_validated_ledger_sequence},
        transaction::exceptions::XRPLSignTransactionException,
    },
    core::{
        addresscodec::{is_valid_xaddress, xaddress_to_classic_address},
        binarycodec::{encode, encode_for_multisigning, encode_for_signing},
        keypairs::sign as keypairs_sign,
    },
    models::{
        requests::{server_state::ServerState, submit::Submit},
        results::{server_state::ServerState as ServerStateResult, submit::Submit as SubmitResult},
        transactions::{
            exceptions::XRPLTransactionFieldException, Signer, Transaction, TransactionType,
        },
        Model, XRPAmount, XRPLModelException,
    },
    utils::transactions::{
        get_transaction_field_value, set_transaction_field_value, validate_transaction_has_field,
    },
    wallet::Wallet,
};

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{borrow::Cow, vec};
use core::convert::TryInto;
use core::fmt::Debug;
use exceptions::XRPLTransactionHelperException;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use strum::IntoEnumIterator;

use super::exceptions::XRPLHelperResult;

const OWNER_RESERVE: &str = "2000000"; // 2 XRP
const RESTRICTED_NETWORKS: u16 = 1024;
const REQUIRED_NETWORKID_VERSION: &str = "1.11.0";
const LEDGER_OFFSET: u8 = 20;

pub fn sign<'a, T, F>(transaction: &mut T, wallet: &Wallet, multisign: bool) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
{
    transaction.validate()?;

    if multisign {
        let serialized_for_signing =
            encode_for_multisigning(transaction, wallet.classic_address.clone().into())?;
        let serialized_bytes = hex::decode(serialized_for_signing)?;
        let signature = keypairs_sign(&serialized_bytes, &wallet.private_key)?;
        let signer = Signer::new(
            wallet.classic_address.clone().into(),
            signature.into(),
            wallet.public_key.clone().into(),
        );
        transaction.get_mut_common_fields().signers = Some(vec![signer]);

        Ok(())
    } else {
        prepare_transaction(transaction, wallet)?;
        let serialized_for_signing = encode_for_signing(transaction)?;
        let serialized_bytes = hex::decode(serialized_for_signing)?;
        let signature = keypairs_sign(&serialized_bytes, &wallet.private_key)?;
        transaction.get_mut_common_fields().txn_signature = Some(signature.into());

        Ok(())
    }
}

pub async fn sign_and_submit<'a, 'b, T, F, C>(
    transaction: &mut T,
    client: &'b C,
    wallet: &Wallet,
    autofill: bool,
    check_fee: bool,
) -> XRPLHelperResult<SubmitResult<'a>>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
    C: XRPLAsyncClient,
{
    if autofill {
        autofill_and_sign(transaction, client, wallet, check_fee).await?;
    } else {
        if check_fee {
            check_txn_fee(transaction, client).await?;
        }
        sign(transaction, wallet, false)?;
    }
    submit(transaction, client).await
}

pub async fn autofill<'a, 'b, F, T, C>(
    transaction: &mut T,
    client: &'b C,
    signers_count: Option<u8>,
) -> XRPLHelperResult<()>
where
    T: Transaction<'a, F> + Model + Clone,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    C: XRPLAsyncClient,
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
            Some(calculate_fee_per_transaction_type(&txn, Some(client), signers_count).await?);
    }
    if txn_common_fields.last_ledger_sequence.is_none() {
        let ledger_sequence = get_latest_validated_ledger_sequence(client).await?;
        txn_common_fields.last_ledger_sequence = Some(ledger_sequence + LEDGER_OFFSET as u32);
    }

    Ok(())
}

pub async fn autofill_and_sign<'a, 'b, T, F, C>(
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
    if check_fee {
        check_txn_fee(transaction, client).await?;
    }
    autofill(transaction, client, None).await?;
    sign(transaction, wallet, false)?;

    Ok(())
}

pub async fn submit<'a, T, F, C>(transaction: &T, client: &C) -> XRPLHelperResult<SubmitResult<'a>>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone + Debug,
    C: XRPLAsyncClient,
{
    transaction.validate()?;
    let txn_blob = encode(transaction)?;
    let req = Submit::new(None, txn_blob.into(), None);
    let res = client.request(req.into()).await?;

    Ok(res.try_into_result::<SubmitResult<'_>>()?)
}

pub async fn calculate_fee_per_transaction_type<'a, 'b, 'c, T, F, C>(
    transaction: &T,
    client: Option<&'b C>,
    signers_count: Option<u8>,
) -> XRPLHelperResult<XRPAmount<'c>>
where
    T: Transaction<'a, F>,
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    C: XRPLAsyncClient,
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
    let mut base_fee_decimal: BigDecimal = base_fee.try_into()?;
    if let Some(signers_count) = signers_count {
        let net_fee_decimal: BigDecimal = net_fee.try_into()?;
        let signer_count_fee_decimal: BigDecimal = (1 + signers_count).into();
        base_fee_decimal += &(net_fee_decimal * signer_count_fee_decimal);
    }

    Ok(base_fee_decimal
        .with_scale_round(0, RoundingMode::Down)
        .into())
}

async fn get_owner_reserve_from_response(
    client: &impl XRPLAsyncClient,
) -> XRPLHelperResult<XRPAmount<'_>> {
    let owner_reserve_response = client.request(ServerState::new(None).into()).await?;
    match owner_reserve_response
        .try_into_result::<ServerStateResult<'_>>()?
        .state
        .validated_ledger
    {
        Some(validated_ledger) => Ok(validated_ledger.reserve_base),
        None => Err(XRPLModelException::MissingField("validated_ledger".to_string()).into()),
    }
}

fn calculate_base_fee_for_escrow_finish<'a: 'b, 'b>(
    net_fee: XRPAmount<'a>,
    fulfillment: Option<Cow<str>>,
) -> XRPLHelperResult<XRPAmount<'b>> {
    if let Some(fulfillment) = fulfillment {
        calculate_based_on_fulfillment(fulfillment, net_fee)
    } else {
        Ok(net_fee)
    }
}

fn calculate_based_on_fulfillment<'a>(
    fulfillment: Cow<str>,
    net_fee: XRPAmount<'_>,
) -> XRPLHelperResult<XRPAmount<'a>> {
    let fulfillment_bytes: Vec<u8> = fulfillment.chars().map(|c| c as u8).collect();
    let net_fee_f64: f64 = net_fee.try_into()?;
    let base_fee_string =
        (net_fee_f64 * (33.0 + (fulfillment_bytes.len() as f64 / 16.0))).to_string();
    let base_fee: XRPAmount = base_fee_string.into();
    let base_fee_decimal: BigDecimal = base_fee.try_into()?;

    Ok(base_fee_decimal
        .with_scale_round(0, RoundingMode::Down)
        .into())
}

fn txn_needs_network_id(common_fields: CommonFields<'_>) -> XRPLHelperResult<bool> {
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
            Err(e) => Err(e),
        }
    } else {
        Ok(false)
    }
}

fn is_not_later_rippled_version(source: String, target: String) -> XRPLHelperResult<bool> {
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
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?,
            source_decomp[1]
                .parse::<u8>()
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?,
        );
        let (target_major, target_minor) = (
            target_decomp[0]
                .parse::<u8>()
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?,
            target_decomp[1]
                .parse::<u8>()
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?,
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
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?;
            let target_patch_version = target_patch[0]
                .parse::<u8>()
                .map_err(XRPLTransactionHelperException::ParseRippledVersionError)?;
            if source_patch_version != target_patch_version {
                Ok(source_patch_version < target_patch_version)
            } else if source_patch.len() != target_patch.len() {
                Ok(source_patch.len() < target_patch.len())
            } else if source_patch.len() == 2 {
                if source_patch[1].chars().next().ok_or(
                    XRPLTransactionHelperException::InvalidRippledVersion(
                        "source patch version".into(),
                    ),
                )? != target_patch[1].chars().next().ok_or(
                    XRPLTransactionHelperException::InvalidRippledVersion(
                        "target patch version".into(),
                    ),
                )? {
                    Ok(source_patch[1] < target_patch[1])
                } else if source_patch[1].starts_with('b') {
                    Ok(source_patch[1][1..] < target_patch[1][1..])
                } else {
                    Ok(source_patch[1][2..] < target_patch[1][2..])
                }
            } else {
                Ok(false)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
enum AccountFieldType {
    Account,
    Destination,
}

async fn check_txn_fee<'a, 'b, T, F, C>(transaction: &mut T, client: &'b C) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Model + Serialize + DeserializeOwned + Clone,
    C: XRPLAsyncClient,
{
    // max of xrp_to_drops(0.1) and calculate_fee_per_transaction_type
    let expected_fee = XRPAmount::from("100000")
        .max(calculate_fee_per_transaction_type(transaction, Some(client), None).await?);
    let transaction_fee = transaction
        .get_common_fields()
        .fee
        .clone()
        .unwrap_or(XRPAmount::from("0"));
    if transaction_fee > expected_fee {
        Err(XRPLSignTransactionException::FeeTooHigh(transaction_fee.to_string()).into())
    } else {
        Ok(())
    }
}

fn prepare_transaction<'a, T, F>(transaction: &mut T, wallet: &Wallet) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let commond_fields = transaction.get_mut_common_fields();
    commond_fields.signing_pub_key = Some(wallet.public_key.clone().into());

    validate_account_xaddress(transaction, AccountFieldType::Account)?;
    if validate_transaction_has_field(transaction, "Destination").is_ok() {
        validate_account_xaddress(transaction, AccountFieldType::Destination)?;
    }

    let _ = convert_to_classic_address(transaction, "Unauthorize");
    let _ = convert_to_classic_address(transaction, "Authorize");
    // EscrowCancel, EscrowFinish
    let _ = convert_to_classic_address(transaction, "Owner");
    // SetRegularKey

    let _ = convert_to_classic_address(transaction, "RegularKey");

    Ok(())
}

fn validate_account_xaddress<'a, T, F>(
    prepared_transaction: &mut T,
    account_field: AccountFieldType,
) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let (account_field_name, tag_field_name) = match serde_json::to_string(&account_field) {
        Ok(name) => {
            let name_str = name.as_str().trim();
            if name_str == "\"Account\"" {
                ("Account", "SourceTag")
            } else if name_str == "\"Destination\"" {
                ("Destination", "DestinationTag")
            } else {
                return Err(XRPLTransactionFieldException::UnknownAccountField(
                    name_str.to_string(),
                )
                .into());
            }
        }
        Err(error) => return Err(error.into()),
    };
    let account_address = match account_field {
        AccountFieldType::Account => prepared_transaction.get_common_fields().account.clone(),
        AccountFieldType::Destination => {
            get_transaction_field_value(prepared_transaction, "Destination")?
        }
    };

    if is_valid_xaddress(&account_address) {
        let (address, tag, _) = match xaddress_to_classic_address(&account_address) {
            Ok(t) => t,
            Err(error) => return Err(error.into()),
        };
        validate_transaction_has_field(prepared_transaction, account_field_name)?;
        set_transaction_field_value(prepared_transaction, account_field_name, address)?;

        if validate_transaction_has_field(prepared_transaction, tag_field_name).is_ok()
            && get_transaction_field_value(prepared_transaction, tag_field_name).unwrap_or(Some(0))
                != tag
        {
            Err(XRPLSignTransactionException::TagFieldMismatch(tag_field_name.to_string()).into())
        } else {
            set_transaction_field_value(prepared_transaction, tag_field_name, tag)?;

            Ok(())
        }
    } else {
        Ok(())
    }
}

fn convert_to_classic_address<'a, T, F>(
    transaction: &mut T,
    field_name: &str,
) -> XRPLHelperResult<()>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone,
{
    let address = get_transaction_field_value::<F, _, String>(transaction, field_name)?;
    if is_valid_xaddress(&address) {
        let classic_address = match xaddress_to_classic_address(&address) {
            Ok(t) => t.0,
            Err(error) => return Err(error.into()),
        };
        Ok(set_transaction_field_value(
            transaction,
            field_name,
            classic_address,
        )?)
    } else {
        Ok(())
    }
}

#[cfg(all(feature = "websocket", feature = "std"))]
#[cfg(test)]
mod test_autofill {
    use super::autofill;
    use crate::{
        asynch::{
            clients::{AsyncWebSocketClient, SingleExecutorMutex},
            exceptions::XRPLHelperResult,
        },
        models::{
            transactions::{offer_create::OfferCreate, Transaction},
            IssuedCurrencyAmount, XRPAmount,
        },
    };

    #[tokio::test]
    async fn test_autofill_txn() -> XRPLHelperResult<()> {
        let mut txn = OfferCreate::new(
            "r9mhdWo1NXVZr2pDnCtC1xwxE85kFtSzYR".into(),
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
        let client = AsyncWebSocketClient::<SingleExecutorMutex, _>::open(
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

#[cfg(all(feature = "json-rpc", feature = "std"))]
#[cfg(test)]
mod test_sign {
    use alloc::borrow::Cow;

    use crate::{
        asynch::{
            clients::AsyncJsonRpcClient,
            transaction::{autofill_and_sign, sign},
            wallet::generate_faucet_wallet,
        },
        models::transactions::{account_set::AccountSet, Transaction},
        wallet::Wallet,
    };

    #[tokio::test]
    async fn test_sign() {
        let wallet = Wallet::new("sEdT7wHTCLzDG7ueaw4hroSTBvH7Mk5", 0).unwrap();
        let mut tx = AccountSet::new(
            Cow::from(wallet.classic_address.clone()),
            None,
            Some("10".into()),
            None,
            None,
            None,
            Some(227234),
            None,
            None,
            None,
            None,
            Some("6578616d706c652e636f6d".into()), // "example.com"
            None,
            None,
            None,
            None,
            None,
            None,
        );
        sign(&mut tx, &wallet, false).unwrap();
        let expected_signature: Cow<str> =
            "C3F435CFBFAE996FE297F3A71BEAB68FF5322CBF039E41A9615BC48A59FB4EC\
            5A55F8D4EC0225D47056E02ECCCDF7E8FF5F8B7FAA1EBBCBF7D0491FCB2D98807"
                .into();
        let actual_signature = tx.get_common_fields().txn_signature.as_ref().unwrap();
        assert_eq!(expected_signature, *actual_signature);
    }

    #[tokio::test]
    async fn test_autofill_and_sign() {
        let client = AsyncJsonRpcClient::connect("https://testnet.xrpl-labs.com/".parse().unwrap());
        let wallet = generate_faucet_wallet(&client, None, None, None, None)
            .await
            .unwrap();
        let mut tx = AccountSet::new(
            Cow::from(wallet.classic_address.clone()),
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
            Some("6578616d706c652e636f6d".into()), // "example.com"
            None,
            None,
            None,
            None,
            None,
            None,
        );
        autofill_and_sign(&mut tx, &client, &wallet, true)
            .await
            .unwrap();
        assert!(tx.get_common_fields().sequence.is_some());
        assert!(tx.get_common_fields().txn_signature.is_some());
    }
}
