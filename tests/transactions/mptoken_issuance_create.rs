// xrpl.js reference: packages/xrpl/test/integration/transactions/mptokenIssuanceCreate.test.ts
// rippled spec: XLS-94D (DynamicMPT)
//
// Scenarios:
//   - base:
//       Create an issuance with MaximumAmount + MPTokenMetadata; verify the object
//       is present on the ledger with the correct MaximumAmount.
//
//   - persists_flags_and_immutable_flags:
//       Create with Flags=tfMPTCanTransfer and ImmutableFlags=tifMPTTransferFee;
//       verify that the resulting MPTokenIssuance has lsfMPTCanTransfer set in Flags
//       and lsifMPTTransferFee set in ImmutableFlags, while lsifMPTMetadata is absent.

use crate::common::{generate_funded_wallet, get_client, ledger_accept, with_blockchain_lock};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::ledger::objects::mptoken_issuance::{
    MPTokenIssuanceFlag, MPTokenIssuanceImmutableFlag,
};
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::{CommonFields as RequestCommonFields, RequestMethod};
use xrpl::models::results;
use xrpl::models::transactions::{
    mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
    CommonFields, TransactionType,
};

/// Compute the raw numeric Flags value from the on-ledger object's `Flags` field.
fn parse_lsf_flags(flags_value: u64) -> Vec<MPTokenIssuanceFlag> {
    let bits = flags_value as u32;
    let mut out = Vec::new();
    if bits & 0x00000001 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTLocked);
    }
    if bits & 0x00000002 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTCanLock);
    }
    if bits & 0x00000004 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTRequireAuth);
    }
    if bits & 0x00000008 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTCanEscrow);
    }
    if bits & 0x00000010 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTCanTrade);
    }
    if bits & 0x00000020 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTCanTransfer);
    }
    if bits & 0x00000040 != 0 {
        out.push(MPTokenIssuanceFlag::LsfMPTCanClawback);
    }
    out
}

/// Return true when the ImmutableFlags integer contains the given `lsif*` bit.
fn has_lsif(immutable_flags: Option<u64>, flag: MPTokenIssuanceImmutableFlag) -> bool {
    let bits = immutable_flags.unwrap_or(0) as u32;
    bits & (flag as u32) != 0
}

#[tokio::test]
async fn test_mptoken_issuance_create_base() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        let mut tx = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                ..Default::default()
            },
            // 0x7fffffffffffffff
            maximum_amount: Some("9223372036854775807".into()),
            asset_scale: Some(2),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");
        assert_eq!(
            result.engine_result, "tesSUCCESS",
            "Expected tesSUCCESS but got: {} — {}",
            result.engine_result, result.engine_result_message
        );
        ledger_accept().await;

        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::MptIssuance),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let ao: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("account_objects parse failed");

        assert_eq!(
            ao.account_objects.len(),
            1,
            "Should be exactly one MPTokenIssuance on the ledger"
        );

        let obj = &ao.account_objects[0];
        assert_eq!(
            obj["MaximumAmount"].as_str(),
            Some("9223372036854775807"),
            "MaximumAmount mismatch"
        );
    })
    .await;
}

#[tokio::test]
async fn test_mptoken_issuance_create_persists_flags_and_immutable_flags() {
    with_blockchain_lock(|| async {
        let client = get_client().await;
        let wallet = generate_funded_wallet().await;

        // A capability enabled at create time (tfMPTCanTransfer → lsfMPTCanTransfer) …
        // … plus permanently making TransferFee immutable. All other fields and flags
        // remain mutable by default (XLS-94D "mutable by default, immutable on demand").
        let mut tx = MPTokenIssuanceCreate {
            common_fields: CommonFields {
                account: wallet.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceCreate,
                flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                ..Default::default()
            },
            immutable_flags: Some(vec![MPTokenIssuanceImmutableFlag::LsifMPTTransferFee].into()),
            ..Default::default()
        };

        let result = sign_and_submit(&mut tx, client, &wallet, true, true)
            .await
            .expect("sign_and_submit failed");
        assert_eq!(
            result.engine_result, "tesSUCCESS",
            "Expected tesSUCCESS but got: {} — {}",
            result.engine_result, result.engine_result_message
        );
        // Build the issuance ID from the autofilled Sequence and account.
        let sequence = result.tx_json["Sequence"]
            .as_u64()
            .expect("Sequence missing") as u32;
        let account_id = xrpl::core::addresscodec::decode_classic_address(&wallet.classic_address)
            .expect("decode_classic_address failed");
        let mut id_bytes = [0u8; 24];
        id_bytes[..4].copy_from_slice(&sequence.to_be_bytes());
        id_bytes[4..].copy_from_slice(&account_id);
        let issuance_id = hex::encode_upper(&id_bytes);

        ledger_accept().await;

        let ao_response = client
            .request(
                AccountObjects {
                    common_fields: RequestCommonFields {
                        command: RequestMethod::AccountObjects,
                        id: None,
                    },
                    account: wallet.classic_address.clone().into(),
                    ledger_lookup: None,
                    r#type: Some(AccountObjectType::MptIssuance),
                    deletion_blockers_only: None,
                    limit: None,
                    marker: None,
                }
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let ao: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("account_objects parse failed");

        let issuance = ao
            .account_objects
            .iter()
            .find(|o| {
                // rippled account_objects returns the ID as lowercase "mpt_issuance_id"
                o["mpt_issuance_id"]
                    .as_str()
                    .map(|id| id.eq_ignore_ascii_case(&issuance_id))
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| {
                panic!(
                    "Created MPTokenIssuance ({issuance_id}) not found in account_objects: {ao:?}"
                )
            });

        // Verify Flags: lsfMPTCanTransfer (0x20) must be set.
        let flags_val = issuance["Flags"].as_u64().expect("Flags missing");
        let lsf_flags = parse_lsf_flags(flags_val);
        assert!(
            lsf_flags.contains(&MPTokenIssuanceFlag::LsfMPTCanTransfer),
            "lsfMPTCanTransfer should be set in Flags (got {flags_val:#010X})"
        );

        // Verify ImmutableFlags: lsifMPTTransferFee (0x20000) must be set.
        let immutable_val = issuance["ImmutableFlags"].as_u64();
        assert!(
            has_lsif(
                immutable_val,
                MPTokenIssuanceImmutableFlag::LsifMPTTransferFee
            ),
            "lsifMPTTransferFee should be set in ImmutableFlags (got {immutable_val:?})"
        );

        // lsifMPTMetadata must NOT be set (we never made metadata immutable).
        assert!(
            !has_lsif(immutable_val, MPTokenIssuanceImmutableFlag::LsifMPTMetadata),
            "lsifMPTMetadata should NOT be set in ImmutableFlags"
        );
    })
    .await;
}
