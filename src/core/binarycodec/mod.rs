//! Functions for encoding objects into the XRP Ledger's
//! canonical binary format and decoding them.

pub mod definitions;
pub mod types;

use types::{AccountId, STObject};

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::convert::TryFrom;
use hex::ToHex;
use serde::Serialize;

pub mod binary_wrappers;
pub mod exceptions;
pub(crate) mod test_cases;
pub mod utils;

pub use binary_wrappers::*;

use crate::XRPLSerdeJsonError;

use super::exceptions::XRPLCoreResult;

const TRANSACTION_SIGNATURE_PREFIX: i32 = 0x53545800;
const TRANSACTION_MULTISIG_PREFIX: [u8; 4] = (0x534D5400u32).to_be_bytes();

pub fn encode<T>(signed_transaction: &T) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    serialize_json(signed_transaction, None, None, false)
}

pub fn encode_for_signing<T>(prepared_transaction: &T) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_SIGNATURE_PREFIX.to_be_bytes().as_ref()),
        None,
        true,
    )
}

pub fn encode_for_multisigning<T>(
    prepared_transaction: &T,
    signing_account: Cow<'_, str>,
) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    let signing_account_id = AccountId::try_from(signing_account.as_ref()).unwrap();

    serialize_json(
        prepared_transaction,
        Some(TRANSACTION_MULTISIG_PREFIX.as_ref()),
        Some(signing_account_id.as_ref()),
        true,
    )
}

fn serialize_json<T>(
    prepared_transaction: &T,
    prefix: Option<&[u8]>,
    suffix: Option<&[u8]>,
    signing_only: bool,
) -> XRPLCoreResult<String>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    if let Some(p) = prefix {
        buffer.extend(p);
    }

    let json_value =
        serde_json::to_value(prepared_transaction).map_err(XRPLSerdeJsonError::from)?;
    // dbg!(&json_value);
    let st_object = STObject::try_from_value(json_value, signing_only)?;
    buffer.extend(st_object.as_ref());

    if let Some(s) = suffix {
        buffer.extend(s);
    }
    let hex_string = buffer.encode_hex_upper::<String>();

    Ok(hex_string)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::binarycodec::test_cases::load_additional_tx_fixtures;

    /// Test that transaction encoding matches expected binary from xrpl.js fixtures.
    #[test]
    fn test_encode_additional_fixtures() {
        let fixtures = load_additional_tx_fixtures();
        let total = fixtures.transactions.len();

        println!(
            "\n=== Running {} xrpl.js transaction fixture tests ===\n",
            total
        );

        let mut passed = 0;
        let mut failed = 0;

        for (i, fixture) in fixtures.transactions.iter().enumerate() {
            let result = encode(&fixture.json);

            match result {
                Ok(encoded) => {
                    if encoded.to_uppercase() == fixture.binary.to_uppercase() {
                        println!("  ✓ [{}/{}] {} passed", i + 1, total, fixture.name);
                        passed += 1;
                    } else {
                        println!("  ✗ [{}/{}] {} MISMATCH", i + 1, total, fixture.name);
                        println!("    Expected: {}", fixture.binary);
                        println!("    Got:      {}", encoded);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ [{}/{}] {} FAILED: {:?}", i + 1, total, fixture.name, e);
                    failed += 1;
                }
            }
        }

        println!(
            "\n=== Results: {} passed, {} failed out of {} ===\n",
            passed, failed, total
        );

        if failed > 0 {
            panic!("{} out of {} tests failed", failed, total);
        }
    }
}
