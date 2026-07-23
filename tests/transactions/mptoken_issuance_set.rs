// xrpl.js reference: packages/xrpl/test/integration/transactions/mptokenIssuanceSet.test.ts
// rippled spec: XLS-94D (DynamicMPT)
//
// Scenarios:
//   - base:
//       Create a lockable issuance, then lock it at the issuance level.
//
//   - enables_every_capability_flag_via_set:
//       Create a plain issuance (no capabilities). Verify none are set. Then enable all
//       six capability flags at once via tfMPTSet* Flags. Verify all are now set.
//
//   - rejects_capability_flag_made_immutable_at_create:
//       Create with ImmutableFlags=tifMPTCanLock. Attempting tfMPTSetCanLock returns
//       tecNO_PERMISSION.
//
//   - mutates_transfer_fee:
//       Create with tfMPTCanTransfer. Set TransferFee via MPTokenIssuanceSet. Verify the
//       value is stored on the ledger object.
//
//   - rejects_transfer_fee_when_immutable:
//       Create with tfMPTCanTransfer + ImmutableFlags=tifMPTTransferFee. Attempting to
//       set TransferFee returns tecNO_PERMISSION.
//
//   - makes_transfer_fee_immutable_via_set:
//       Create with tfMPTCanTransfer. Set TransferFee=200 and ImmutableFlags=tifMPTTransferFee
//       in the same transaction. Verify the ledger records both the fee and the immutable bit.
//       Then confirm a subsequent attempt to change TransferFee returns tecNO_PERMISSION.
//
//   - mutates_metadata:
//       Create with MPTokenMetadata. Update it via MPTokenIssuanceSet. Verify new value on
//       ledger.
//
//   - rejects_metadata_when_immutable:
//       Create with MPTokenMetadata + ImmutableFlags=tifMPTMetadata. Attempting to update
//       metadata returns tecNO_PERMISSION.
//
//   - persists_domain_id_at_create:
//       Create with tfMPTRequireAuth + DomainID. Verify DomainID is stored.
//
//   - updates_domain_id_via_set:
//       Create with tfMPTRequireAuth + firstDomainID. Update to secondDomainID. Verify.
//
//   - rejects_domain_id_without_require_auth:
//       Create without tfMPTRequireAuth. Attempt to set DomainID → tecNO_PERMISSION.

use crate::common::{
    generate_funded_wallet, get_client, ledger_accept, submit_tx, test_transaction,
    with_blockchain_lock, SubmitOptions,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::asynch::transaction::sign_and_submit;
use xrpl::models::ledger::objects::mptoken_issuance::MPTokenIssuanceImmutableFlag;
use xrpl::models::requests::account_objects::{AccountObjectType, AccountObjects};
use xrpl::models::requests::{CommonFields as RequestCommonFields, RequestMethod};
use xrpl::models::results;
use xrpl::models::transactions::{
    mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
    mptoken_issuance_set::{MPTokenIssuanceSet, MPTokenIssuanceSetFlag},
    permissioned_domain_set::PermissionedDomainSet,
    CommonFields, Credential, TransactionType,
};
use xrpl::wallet::Wallet;

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Return true when the ImmutableFlags integer contains the given `lsif*` bit.
fn has_lsif(immutable_flags: Option<u64>, flag: MPTokenIssuanceImmutableFlag) -> bool {
    let bits = immutable_flags.unwrap_or(0) as u32;
    bits & (flag as u32) != 0
}

/// Create a plain MPTokenIssuance (no capabilities set) and return its ID.
async fn create_plain_issuance(wallet: &Wallet) -> String {
    let client = get_client().await;
    let mut tx = MPTokenIssuanceCreate {
        common_fields: CommonFields {
            account: wallet.classic_address.clone().into(),
            transaction_type: TransactionType::MPTokenIssuanceCreate,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = sign_and_submit(&mut tx, client, wallet, true, true)
        .await
        .expect("create_plain_issuance: sign_and_submit failed");
    assert_eq!(
        result.engine_result, "tesSUCCESS",
        "create_plain_issuance: {}",
        result.engine_result_message
    );
    let sequence = result.tx_json["Sequence"]
        .as_u64()
        .expect("Sequence missing") as u32;
    let account_id = xrpl::core::addresscodec::decode_classic_address(&wallet.classic_address)
        .expect("decode_classic_address failed");
    let mut id_bytes = [0u8; 24];
    id_bytes[..4].copy_from_slice(&sequence.to_be_bytes());
    id_bytes[4..].copy_from_slice(&account_id);
    ledger_accept().await;
    hex::encode_upper(&id_bytes)
}

/// Create an MPTokenIssuance with custom settings and return its ID.
async fn create_issuance_with(wallet: &Wallet, mut tx: MPTokenIssuanceCreate<'_>) -> String {
    let client = get_client().await;
    let result = sign_and_submit(&mut tx, client, wallet, true, true)
        .await
        .expect("create_issuance_with: sign_and_submit failed");
    assert_eq!(
        result.engine_result, "tesSUCCESS",
        "create_issuance_with: {}",
        result.engine_result_message
    );
    let sequence = result.tx_json["Sequence"]
        .as_u64()
        .expect("Sequence missing") as u32;
    let account_id = xrpl::core::addresscodec::decode_classic_address(&wallet.classic_address)
        .expect("decode_classic_address failed");
    let mut id_bytes = [0u8; 24];
    id_bytes[..4].copy_from_slice(&sequence.to_be_bytes());
    id_bytes[4..].copy_from_slice(&account_id);
    ledger_accept().await;
    hex::encode_upper(&id_bytes)
}

/// Read back an MPTokenIssuance object from account_objects.
async fn read_mpt_issuance(wallet: &Wallet, issuance_id: &str) -> serde_json::Value {
    let client = get_client().await;
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
        .expect("read_mpt_issuance: account_objects request failed");
    let ao: results::account_objects::AccountObjects<'_> = ao_response
        .try_into()
        .expect("read_mpt_issuance: parse failed");
    ao.account_objects
        .iter()
        .find(|o| {
            // rippled account_objects returns the ID as lowercase "mpt_issuance_id"
            o["mpt_issuance_id"]
                .as_str()
                .map(|id| id.eq_ignore_ascii_case(issuance_id))
                .unwrap_or(false)
        })
        .cloned()
        .unwrap_or_else(|| panic!("MPTokenIssuance {issuance_id} not found"))
}

/// Create a PermissionedDomain and return its on-ledger index (DomainID).
async fn create_permissioned_domain(wallet: &Wallet) -> String {
    let client = get_client().await;
    let cred_type_hex = "50617373706F7274"; // hex("Passport")
    let mut pd_set = PermissionedDomainSet {
        common_fields: CommonFields {
            account: wallet.classic_address.clone().into(),
            transaction_type: TransactionType::PermissionedDomainSet,
            ..Default::default()
        },
        domain_id: None,
        accepted_credentials: vec![Credential {
            issuer: wallet.classic_address.clone(),
            credential_type: cred_type_hex.to_string(),
        }],
    };
    let result = sign_and_submit(&mut pd_set, client, wallet, true, true)
        .await
        .expect("create_permissioned_domain: sign_and_submit failed");
    assert_eq!(
        result.engine_result, "tesSUCCESS",
        "create_permissioned_domain: {}",
        result.engine_result_message
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
                r#type: Some(AccountObjectType::PermissionedDomain),
                deletion_blockers_only: None,
                limit: None,
                marker: None,
            }
            .into(),
        )
        .await
        .expect("create_permissioned_domain: account_objects request failed");
    let ao: results::account_objects::AccountObjects<'_> = ao_response
        .try_into()
        .expect("create_permissioned_domain: parse failed");
    assert!(
        !ao.account_objects.is_empty(),
        "No PermissionedDomain found after creation"
    );
    ao.account_objects.last().expect("empty")["index"]
        .as_str()
        .expect("index field missing")
        .to_string()
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mptoken_issuance_set_base() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        // Create a lockable issuance (TfMPTCanLock helper sets this flag).
        let issuance_id = crate::common::create_mptoken_issuance(&issuer).await;

        // Lock the entire issuance at the issuance level.
        let mut lock_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            ..Default::default()
        };
        test_transaction(&mut lock_tx, &issuer).await;
    })
    .await;
}

/// Mirrors JS: "enables every capability flag one-way via MPTokenIssuanceSet (XLS-94D)"
#[tokio::test]
async fn test_mptoken_issuance_set_enables_every_capability_flag() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        // Create a plain issuance — no capabilities set yet (mutable by default under XLS-94D).
        let issuance_id = create_plain_issuance(&issuer).await;
        let before = read_mpt_issuance(&issuer, &issuance_id).await;
        let flags_before = before["Flags"].as_u64().unwrap_or(0) as u32;

        // None of the capability flags should be set on a fresh issuance.
        assert_eq!(
            flags_before & 0x00000002,
            0,
            "lsfMPTCanLock unexpectedly set"
        );
        assert_eq!(
            flags_before & 0x00000004,
            0,
            "lsfMPTRequireAuth unexpectedly set"
        );
        assert_eq!(
            flags_before & 0x00000008,
            0,
            "lsfMPTCanEscrow unexpectedly set"
        );
        assert_eq!(
            flags_before & 0x00000010,
            0,
            "lsfMPTCanTrade unexpectedly set"
        );
        assert_eq!(
            flags_before & 0x00000020,
            0,
            "lsfMPTCanTransfer unexpectedly set"
        );
        assert_eq!(
            flags_before & 0x00000040,
            0,
            "lsfMPTCanClawback unexpectedly set"
        );

        // Enable all six capability flags in a single MPTokenIssuanceSet transaction.
        let mut enable_all_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![
                    MPTokenIssuanceSetFlag::TfMPTSetCanLock,
                    MPTokenIssuanceSetFlag::TfMPTSetRequireAuth,
                    MPTokenIssuanceSetFlag::TfMPTSetCanEscrow,
                    MPTokenIssuanceSetFlag::TfMPTSetCanTrade,
                    MPTokenIssuanceSetFlag::TfMPTSetCanTransfer,
                    MPTokenIssuanceSetFlag::TfMPTSetCanClawback,
                ]
                .into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            ..Default::default()
        };
        test_transaction(&mut enable_all_tx, &issuer).await;

        let after = read_mpt_issuance(&issuer, &issuance_id).await;
        let flags_after = after["Flags"].as_u64().unwrap_or(0) as u32;

        assert_ne!(flags_after & 0x00000002, 0, "lsfMPTCanLock should be set");
        assert_ne!(
            flags_after & 0x00000004,
            0,
            "lsfMPTRequireAuth should be set"
        );
        assert_ne!(flags_after & 0x00000008, 0, "lsfMPTCanEscrow should be set");
        assert_ne!(flags_after & 0x00000010, 0, "lsfMPTCanTrade should be set");
        assert_ne!(
            flags_after & 0x00000020,
            0,
            "lsfMPTCanTransfer should be set"
        );
        assert_ne!(
            flags_after & 0x00000040,
            0,
            "lsfMPTCanClawback should be set"
        );
    })
    .await;
}

/// Mirrors JS: "rejects enabling a capability that was made immutable at create time"
#[tokio::test]
async fn test_mptoken_issuance_set_rejects_capability_made_immutable() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        // Create with ImmutableFlags=tifMPTCanLock — permanently prevent enabling CanLock.
        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    ..Default::default()
                },
                immutable_flags: Some(vec![MPTokenIssuanceImmutableFlag::LsifMPTCanLock].into()),
                ..Default::default()
            },
        )
        .await;

        let mut set_can_lock_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTSetCanLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            ..Default::default()
        };
        let engine_result = submit_tx(
            &mut set_can_lock_tx,
            SubmitOptions {
                wallet: &issuer,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            engine_result, "tecNO_PERMISSION",
            "Enabling an immutable capability flag must return tecNO_PERMISSION"
        );
        ledger_accept().await;
    })
    .await;
}

/// Mirrors JS: "mutates TransferFee via MPTokenIssuanceSet (mutable by default under XLS-94D)"
#[tokio::test]
async fn test_mptoken_issuance_set_mutates_transfer_fee() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

        let mut set_fee_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            transfer_fee: Some(200),
            ..Default::default()
        };
        test_transaction(&mut set_fee_tx, &issuer).await;

        let issuance = read_mpt_issuance(&issuer, &issuance_id).await;
        assert_eq!(
            issuance["TransferFee"].as_u64(),
            Some(200),
            "TransferFee should be 200 after MPTokenIssuanceSet"
        );
    })
    .await;
}

/// Mirrors JS: "rejects TransferFee mutation … when TransferFee was made immutable at create time"
#[tokio::test]
async fn test_mptoken_issuance_set_rejects_transfer_fee_when_immutable() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                    ..Default::default()
                },
                immutable_flags: Some(
                    vec![MPTokenIssuanceImmutableFlag::LsifMPTTransferFee].into(),
                ),
                ..Default::default()
            },
        )
        .await;

        let mut update_fee_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            transfer_fee: Some(100),
            ..Default::default()
        };
        let engine_result = submit_tx(
            &mut update_fee_tx,
            SubmitOptions {
                wallet: &issuer,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            engine_result, "tecNO_PERMISSION",
            "Modifying an immutable TransferFee must return tecNO_PERMISSION"
        );
        ledger_accept().await;
    })
    .await;
}

/// Mirrors JS: "makes TransferFee immutable via MPTokenIssuanceSet ImmutableFlags"
#[tokio::test]
async fn test_mptoken_issuance_set_makes_transfer_fee_immutable() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    flags: vec![MPTokenIssuanceCreateFlag::TfMPTCanTransfer].into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

        // Set TransferFee=200 and permanently lock it in the same transaction.
        let mut set_and_lock_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            transfer_fee: Some(200),
            immutable_flags: Some(vec![MPTokenIssuanceImmutableFlag::LsifMPTTransferFee].into()),
            ..Default::default()
        };
        test_transaction(&mut set_and_lock_tx, &issuer).await;

        let issuance = read_mpt_issuance(&issuer, &issuance_id).await;
        assert_eq!(
            issuance["TransferFee"].as_u64(),
            Some(200),
            "TransferFee should be 200"
        );
        assert!(
            has_lsif(
                issuance["ImmutableFlags"].as_u64(),
                MPTokenIssuanceImmutableFlag::LsifMPTTransferFee
            ),
            "lsifMPTTransferFee should be set in ImmutableFlags"
        );

        // A subsequent attempt to change the (now-immutable) TransferFee must be rejected.
        let mut reject_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            transfer_fee: Some(100),
            ..Default::default()
        };
        let engine_result = submit_tx(
            &mut reject_tx,
            SubmitOptions {
                wallet: &issuer,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            engine_result, "tecNO_PERMISSION",
            "Modifying an immutable TransferFee must return tecNO_PERMISSION"
        );
        ledger_accept().await;
    })
    .await;
}

/// Mirrors JS: "mutates MPTokenMetadata via MPTokenIssuanceSet (mutable by default under XLS-94D)"
#[tokio::test]
async fn test_mptoken_issuance_set_mutates_metadata() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        let initial_metadata = "DEADBEEF";
        let updated_metadata = "CAFEBABE";

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    ..Default::default()
                },
                mptoken_metadata: Some(initial_metadata.into()),
                ..Default::default()
            },
        )
        .await;

        let mut update_meta_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            mptoken_metadata: Some(updated_metadata.into()),
            ..Default::default()
        };
        test_transaction(&mut update_meta_tx, &issuer).await;

        let issuance = read_mpt_issuance(&issuer, &issuance_id).await;
        assert_eq!(
            issuance["MPTokenMetadata"].as_str(),
            Some(updated_metadata),
            "MPTokenMetadata should be updated to {updated_metadata}"
        );
    })
    .await;
}

/// Mirrors JS: "rejects MPTokenMetadata mutation … when MPTokenMetadata was made immutable at create time"
#[tokio::test]
async fn test_mptoken_issuance_set_rejects_metadata_when_immutable() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    ..Default::default()
                },
                mptoken_metadata: Some("DEADBEEF".into()),
                immutable_flags: Some(vec![MPTokenIssuanceImmutableFlag::LsifMPTMetadata].into()),
                ..Default::default()
            },
        )
        .await;

        let mut update_meta_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            mptoken_metadata: Some("CAFEBABE".into()),
            ..Default::default()
        };
        let engine_result = submit_tx(
            &mut update_meta_tx,
            SubmitOptions {
                wallet: &issuer,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            engine_result, "tecNO_PERMISSION",
            "Modifying immutable MPTokenMetadata must return tecNO_PERMISSION"
        );
        ledger_accept().await;
    })
    .await;
}

/// Mirrors JS: "persists DomainID on the MPTokenIssuance ledger object when set at create time"
#[tokio::test]
async fn test_mptoken_issuance_persists_domain_id_at_create() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let domain_id = create_permissioned_domain(&issuer).await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    flags: vec![MPTokenIssuanceCreateFlag::TfMPTRequireAuth].into(),
                    ..Default::default()
                },
                domain_id: Some(domain_id.clone().into()),
                ..Default::default()
            },
        )
        .await;

        let issuance = read_mpt_issuance(&issuer, &issuance_id).await;
        assert_eq!(
            issuance["DomainID"].as_str(),
            Some(domain_id.as_str()),
            "DomainID should be stored on the MPTokenIssuance object"
        );
    })
    .await;
}

/// Mirrors JS: "updates DomainID on the MPTokenIssuance ledger object via MPTokenIssuanceSet"
#[tokio::test]
async fn test_mptoken_issuance_set_updates_domain_id() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let first_domain_id = create_permissioned_domain(&issuer).await;
        let second_domain_id = create_permissioned_domain(&issuer).await;

        let issuance_id = create_issuance_with(
            &issuer,
            MPTokenIssuanceCreate {
                common_fields: CommonFields {
                    account: issuer.classic_address.clone().into(),
                    transaction_type: TransactionType::MPTokenIssuanceCreate,
                    flags: vec![MPTokenIssuanceCreateFlag::TfMPTRequireAuth].into(),
                    ..Default::default()
                },
                domain_id: Some(first_domain_id.clone().into()),
                ..Default::default()
            },
        )
        .await;

        let mut update_domain_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            domain_id: Some(second_domain_id.clone().into()),
            ..Default::default()
        };
        test_transaction(&mut update_domain_tx, &issuer).await;

        let issuance = read_mpt_issuance(&issuer, &issuance_id).await;
        assert_eq!(
            issuance["DomainID"].as_str(),
            Some(second_domain_id.as_str()),
            "DomainID should be updated to secondDomainID"
        );
    })
    .await;
}

/// Mirrors JS: "rejects DomainID mutation … on an issuance created without tfMPTRequireAuth"
#[tokio::test]
async fn test_mptoken_issuance_set_rejects_domain_id_without_require_auth() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let domain_id = create_permissioned_domain(&issuer).await;

        // Create without tfMPTRequireAuth.
        let issuance_id = create_plain_issuance(&issuer).await;

        let mut set_domain_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            domain_id: Some(domain_id.into()),
            ..Default::default()
        };
        let engine_result = submit_tx(
            &mut set_domain_tx,
            SubmitOptions {
                wallet: &issuer,
                autofill: true,
                check_fee: true,
            },
        )
        .await;
        assert_eq!(
            engine_result, "tecNO_PERMISSION",
            "Setting DomainID without tfMPTRequireAuth must return tecNO_PERMISSION"
        );
        ledger_accept().await;
    })
    .await;
}
