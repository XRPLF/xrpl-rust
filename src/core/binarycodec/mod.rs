//! Functions for encoding objects into the XRP Ledger's
//! canonical binary format and decoding them.

use super::types::{AccountId, STObject};
use crate::{models::transactions::Transaction, Err};

use alloc::{borrow::Cow, string::String, vec::Vec};
use anyhow::Result;
use core::{convert::TryFrom, fmt::Debug};
use hex::ToHex;
use serde::{de::DeserializeOwned, Serialize};
use strum::IntoEnumIterator;

pub mod binary_wrappers;
pub mod exceptions;
pub(crate) mod test_cases;
pub mod utils;

pub use binary_wrappers::*;

const TRANSACTION_SIGNATURE_PREFIX: i32 = 0x53545800;
const TRANSACTION_MULTISIG_PREFIX: i32 = 0x534D5400;

pub fn encode<'a, T, F>(signed_transaction: &T) -> Result<String>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone + Debug,
{
    serialize_json(signed_transaction, None, None, false)
}

pub fn encode_for_signing<'a, T, F>(prepared_transaction: &T) -> Result<String>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone + Debug,
{
    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_SIGNATURE_PREFIX.to_be_bytes().as_ref()),
        None,
        true,
    )
}

pub fn encode_for_multisigning<'a, T, F>(
    prepared_transaction: &T,
    signing_account: Cow<'a, str>,
) -> Result<String>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone + Debug,
{
    let signing_account_id = AccountId::try_from(signing_account.as_ref()).unwrap();

    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_MULTISIG_PREFIX.to_be_bytes().as_ref()),
        Some(signing_account_id.as_ref()),
        true,
    )
}

fn serialize_json<'a, T, F>(
    prepared_transaction: &T,
    prefix: Option<&[u8]>,
    suffix: Option<&[u8]>,
    signing_only: bool,
) -> Result<String>
where
    F: IntoEnumIterator + Serialize + Debug + PartialEq,
    T: Transaction<'a, F> + Serialize + DeserializeOwned + Clone + Debug,
{
    let mut buffer = Vec::new();
    if let Some(p) = prefix {
        buffer.extend(p);
    }

    let json_value = match serde_json::to_value(prepared_transaction) {
        Ok(v) => v,
        Err(e) => {
            return Err!(e);
        }
    };
    let st_object = STObject::try_from_value(json_value, signing_only)?;
    buffer.extend(st_object.as_ref());

    if let Some(s) = suffix {
        buffer.extend(s);
    }
    let hex_string = buffer.encode_hex_upper::<String>();

    Ok(hex_string)
}
