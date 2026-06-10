// MPT end-to-end integration tests.
//
// These mirror the MPT-specific scenarios covered by xrpl.js integration tests:
// - MPTokenIssuanceCreate stores maximum amount and metadata on the ledger
// - MPTokenAuthorize holder opt-in / issuer authorization / holder unauthorize lifecycle
// - MPTokenIssuanceDestroy removes the issuance ledger object
// - Payment can deliver MPT and updates OutstandingAmount
// - Clawback can claw back MPT from a holder and updates the holder balance

use crate::common::{
    assert_transaction_engine_result, generate_funded_wallet, get_client, test_transaction,
    with_blockchain_lock,
};
use serde_json::{json, Value};
use xrpl::{
    asynch::clients::XRPLAsyncClient,
    models::{
        requests::{
            account_objects::{AccountObjectType, AccountObjects},
            CommonFields as RequestCommonFields, RequestMethod,
        },
        results,
        transactions::{
            clawback::Clawback,
            mptoken_authorize::{MPTokenAuthorize, MPTokenAuthorizeFlag},
            mptoken_issuance_create::{MPTokenIssuanceCreate, MPTokenIssuanceCreateFlag},
            mptoken_issuance_destroy::MPTokenIssuanceDestroy,
            mptoken_issuance_set::{MPTokenIssuanceSet, MPTokenIssuanceSetFlag},
            payment::Payment,
            CommonFields, TransactionType,
        },
        Amount, MPTAmount,
    },
    wallet::Wallet,
};

fn mpt_issuance_id(wallet: &Wallet, sequence: u32) -> String {
    let account_id = xrpl::core::addresscodec::decode_classic_address(&wallet.classic_address)
        .expect("failed to decode classic address");
    let mut id_bytes = Vec::with_capacity(24);
    id_bytes.extend_from_slice(&sequence.to_be_bytes());
    id_bytes.extend_from_slice(&account_id);
    hex::encode_upper(&id_bytes)
}

#[derive(Default)]
struct IssuanceOptions {
    flags: Vec<MPTokenIssuanceCreateFlag>,
    maximum_amount: Option<String>,
    asset_scale: Option<u8>,
    metadata: Option<String>,
}

impl IssuanceOptions {
    fn with_flags(mut self, flags: impl IntoIterator<Item = MPTokenIssuanceCreateFlag>) -> Self {
        self.flags = flags.into_iter().collect();
        self
    }

    fn with_maximum_amount(mut self, maximum_amount: impl Into<String>) -> Self {
        self.maximum_amount = Some(maximum_amount.into());
        self
    }

    fn with_asset_scale(mut self, asset_scale: u8) -> Self {
        self.asset_scale = Some(asset_scale);
        self
    }

    fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }
}

async fn create_issuance(issuer: &Wallet, options: IssuanceOptions) -> String {
    let mut tx = MPTokenIssuanceCreate {
        common_fields: CommonFields {
            account: issuer.classic_address.clone().into(),
            transaction_type: TransactionType::MPTokenIssuanceCreate,
            flags: options.flags.into(),
            ..Default::default()
        },
        asset_scale: options.asset_scale,
        maximum_amount: options.maximum_amount.map(Into::into),
        mptoken_metadata: options.metadata.map(Into::into),
        ..Default::default()
    };

    test_transaction(&mut tx, issuer).await;
    let sequence = tx
        .common_fields
        .sequence
        .expect("MPTokenIssuanceCreate sequence missing after autofill");
    mpt_issuance_id(issuer, sequence)
}

async fn account_objects(account: &str, object_type: AccountObjectType) -> Vec<Value> {
    let client = get_client().await;
    let response = client
        .request(
            AccountObjects {
                common_fields: RequestCommonFields {
                    command: RequestMethod::AccountObjects,
                    id: None,
                },
                account: account.into(),
                ledger_lookup: None,
                r#type: Some(object_type),
                deletion_blockers_only: None,
                limit: None,
                marker: None,
            }
            .into(),
        )
        .await
        .expect("account_objects request failed");

    let result: results::account_objects::AccountObjects<'_> = response
        .try_into()
        .expect("failed to parse account_objects result");
    result.account_objects.into_owned()
}

async fn mptoken_ledger_entry(mpt_issuance_id: &str, account: &str) -> Value {
    let response: Value = reqwest::Client::new()
        .post(crate::common::constants::STANDALONE_URL)
        .json(&json!({
            "method": "ledger_entry",
            "params": [{
                "mptoken": {
                    "mpt_issuance_id": mpt_issuance_id,
                    "account": account,
                }
            }]
        }))
        .send()
        .await
        .expect("ledger_entry request failed")
        .json()
        .await
        .expect("ledger_entry JSON parse failed");

    response["result"]["node"].clone()
}

fn metadata_hex(metadata: &Value) -> String {
    hex::encode_upper(
        serde_json::to_string(metadata)
            .expect("metadata JSON serialization should be deterministic")
            .as_bytes(),
    )
}

fn mpt_payment(
    source: &Wallet,
    destination: &Wallet,
    value: &str,
    issuance_id: &str,
) -> Payment<'static> {
    Payment {
        common_fields: CommonFields {
            account: source.classic_address.clone().into(),
            transaction_type: TransactionType::Payment,
            ..Default::default()
        },
        amount: Amount::MPTAmount(MPTAmount::new(
            value.to_owned().into(),
            issuance_id.to_owned().into(),
        )),
        destination: destination.classic_address.clone().into(),
        ..Default::default()
    }
}

fn mpt_clawback(
    issuer: &Wallet,
    holder: &Wallet,
    value: &str,
    issuance_id: &str,
) -> Clawback<'static> {
    Clawback {
        common_fields: CommonFields {
            account: issuer.classic_address.clone().into(),
            transaction_type: TransactionType::Clawback,
            ..Default::default()
        },
        amount: Amount::MPTAmount(MPTAmount::new(
            value.to_owned().into(),
            issuance_id.to_owned().into(),
        )),
        holder: Some(holder.classic_address.clone().into()),
    }
}

#[tokio::test]
async fn test_mptoken_issuance_create_stores_metadata() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let metadata = json!({
            "ticker": "TBILL",
            "name": "T-Bill Yield Token",
            "desc": "A yield-bearing stablecoin backed by short-term U.S. Treasuries and money market instruments.",
            "icon": "example.org/tbill-icon.png",
            "asset_class": "rwa",
            "asset_subclass": "treasury",
            "issuer_name": "Example Yield Co.",
            "uris": [
                {"uri": "exampleyield.co/tbill", "category": "website", "title": "Product Page"},
                {"uri": "exampleyield.co/docs", "category": "docs", "title": "Yield Token Docs"}
            ],
            "additional_info": {
                "interest_rate": "5.00%",
                "interest_type": "variable",
                "yield_source": "U.S. Treasury Bills",
                "maturity_date": "2045-06-30",
                "cusip": "912796RX0"
            }
        });
        let encoded_metadata = metadata_hex(&metadata);

        create_issuance(
            &issuer,
            IssuanceOptions::default()
                .with_maximum_amount("9223372036854775807")
                .with_asset_scale(2)
                .with_metadata(encoded_metadata.clone()),
        )
        .await;

        let objects = account_objects(&issuer.classic_address, AccountObjectType::MptIssuance).await;
        assert_eq!(objects.len(), 1, "should be exactly one issuance on the ledger");
        assert_eq!(objects[0]["MaximumAmount"], "9223372036854775807");
        assert_eq!(objects[0]["MPTokenMetadata"], encoded_metadata);

        let decoded_metadata = String::from_utf8(
            hex::decode(objects[0]["MPTokenMetadata"].as_str().expect("metadata missing"))
                .expect("metadata should be valid hex"),
        )
        .expect("metadata should be UTF-8 JSON");
        let decoded_json: Value = serde_json::from_str(&decoded_metadata)
            .expect("metadata should decode to JSON");
        assert_eq!(decoded_json, metadata);
    })
    .await;
}

#[tokio::test]
async fn test_mptoken_authorize_lifecycle() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(
            &issuer,
            IssuanceOptions::default().with_flags([MPTokenIssuanceCreateFlag::TfMPTRequireAuth]),
        )
        .await;

        let issuer_objects =
            account_objects(&issuer.classic_address, AccountObjectType::MptIssuance).await;
        assert_eq!(
            issuer_objects.len(),
            1,
            "should be exactly one issuance on the ledger"
        );

        let mut holder_opt_in = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut holder_opt_in, &holder).await;

        let holder_objects =
            account_objects(&holder.classic_address, AccountObjectType::Mptoken).await;
        assert_eq!(
            holder_objects.len(),
            1,
            "holder owns one MPToken object on the ledger"
        );

        let mut issuer_authorize = MPTokenAuthorize {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: Some(holder.classic_address.clone().into()),
        };
        test_transaction(&mut issuer_authorize, &issuer).await;

        let mut holder_unauthorize = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                flags: vec![MPTokenAuthorizeFlag::TfMPTUnauthorize].into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
            holder: None,
        };
        test_transaction(&mut holder_unauthorize, &holder).await;

        let holder_objects =
            account_objects(&holder.classic_address, AccountObjectType::Mptoken).await;
        assert_eq!(
            holder_objects.len(),
            0,
            "holder owns no MPToken objects after unauthorize"
        );
    })
    .await;
}

#[tokio::test]
async fn test_mpt_require_auth_rejects_payment_before_issuer_authorization() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(
            &issuer,
            IssuanceOptions::default().with_flags([MPTokenIssuanceCreateFlag::TfMPTRequireAuth]),
        )
        .await;

        let mut holder_opt_in = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut holder_opt_in, &holder).await;

        let mut unauthorized_payment = mpt_payment(&issuer, &holder, "1", &issuance_id);
        assert_transaction_engine_result(&mut unauthorized_payment, &issuer, "tecNO_AUTH").await;

        let mut issuer_authorize = MPTokenAuthorize {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: Some(holder.classic_address.clone().into()),
        };
        test_transaction(&mut issuer_authorize, &issuer).await;

        let mut authorized_payment = mpt_payment(&issuer, &holder, "1", &issuance_id);
        test_transaction(&mut authorized_payment, &issuer).await;
    })
    .await;
}

#[tokio::test]
async fn test_mpt_global_lock_rejects_holder_transfer() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let recipient = generate_funded_wallet().await;
        let issuance_id = create_issuance(
            &issuer,
            IssuanceOptions::default().with_flags([
                MPTokenIssuanceCreateFlag::TfMPTCanLock,
                MPTokenIssuanceCreateFlag::TfMPTCanTransfer,
            ]),
        )
        .await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut recipient_auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: recipient.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut recipient_auth_tx, &recipient).await;

        let mut initial_payment = mpt_payment(&issuer, &holder, "1", &issuance_id);
        test_transaction(&mut initial_payment, &issuer).await;

        let mut lock_tx = MPTokenIssuanceSet {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceSet,
                flags: vec![MPTokenIssuanceSetFlag::TfMPTLock].into(),
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut lock_tx, &issuer).await;

        let mut locked_payment = mpt_payment(&holder, &recipient, "1", &issuance_id);
        assert_transaction_engine_result(&mut locked_payment, &holder, "tecLOCKED").await;
    })
    .await;
}

#[tokio::test]
async fn test_mpt_maximum_amount_rejects_excess_outstanding() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id =
            create_issuance(&issuer, IssuanceOptions::default().with_maximum_amount("1")).await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut capped_payment = mpt_payment(&issuer, &holder, "1", &issuance_id);
        test_transaction(&mut capped_payment, &issuer).await;

        let mut excess_payment = mpt_payment(&issuer, &holder, "1", &issuance_id);
        assert_transaction_engine_result(&mut excess_payment, &issuer, "tecPATH_PARTIAL").await;
    })
    .await;
}

#[tokio::test]
async fn test_mptoken_issuance_destroy_removes_issuance() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let objects =
            account_objects(&issuer.classic_address, AccountObjectType::MptIssuance).await;
        assert_eq!(
            objects.len(),
            1,
            "should be exactly one issuance on the ledger"
        );

        let mut destroy_tx = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
        };
        test_transaction(&mut destroy_tx, &issuer).await;

        let objects =
            account_objects(&issuer.classic_address, AccountObjectType::MptIssuance).await;
        assert_eq!(objects.len(), 0, "should be zero issuances on the ledger");
    })
    .await;
}

#[tokio::test]
async fn test_mptoken_issuance_destroy_rejects_outstanding_tokens() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut payment_tx = mpt_payment(&issuer, &holder, "1", &issuance_id);
        test_transaction(&mut payment_tx, &issuer).await;

        let mut destroy_tx = MPTokenIssuanceDestroy {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenIssuanceDestroy,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.into(),
        };
        assert_transaction_engine_result(&mut destroy_tx, &issuer, "tecHAS_OBLIGATIONS").await;
    })
    .await;
}

#[tokio::test]
async fn test_mpt_payment_updates_outstanding_amount() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let holder_objects =
            account_objects(&holder.classic_address, AccountObjectType::Mptoken).await;
        assert_eq!(
            holder_objects.len(),
            1,
            "holder owns one MPToken object on the ledger"
        );

        let mut pay_tx = mpt_payment(&issuer, &holder, "100", &issuance_id);
        test_transaction(&mut pay_tx, &issuer).await;

        let issuer_objects =
            account_objects(&issuer.classic_address, AccountObjectType::MptIssuance).await;
        assert_eq!(issuer_objects[0]["OutstandingAmount"], "100");
    })
    .await;
}

#[tokio::test]
async fn test_mpt_holder_transfer_requires_transfer_flag() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let recipient = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let mut holder_auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut holder_auth_tx, &holder).await;

        let mut recipient_auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: recipient.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut recipient_auth_tx, &recipient).await;

        let mut issuer_payment = mpt_payment(&issuer, &holder, "10", &issuance_id);
        test_transaction(&mut issuer_payment, &issuer).await;

        let mut holder_transfer = mpt_payment(&holder, &recipient, "1", &issuance_id);
        assert_transaction_engine_result(&mut holder_transfer, &holder, "tecNO_AUTH").await;
    })
    .await;
}

#[tokio::test]
async fn test_mpt_payment_amount_binary_codec_round_trip() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut pay_tx = mpt_payment(&issuer, &holder, "42", &issuance_id);

        let unsigned_hex = xrpl::core::binarycodec::encode(&pay_tx)
            .expect("MPT Payment amount should encode before signing");
        assert!(
            unsigned_hex.contains(&issuance_id),
            "encoded MPT Payment should contain the MPTokenIssuanceID"
        );

        test_transaction(&mut pay_tx, &issuer).await;

        let signed_hex = xrpl::core::binarycodec::encode(&pay_tx)
            .expect("signed MPT Payment amount should encode");
        assert!(
            signed_hex.contains(&issuance_id),
            "signed encoded MPT Payment should contain the MPTokenIssuanceID"
        );
    })
    .await;
}

#[tokio::test]
async fn test_mpt_clawback_rejects_issuance_without_clawback_flag() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(&issuer, IssuanceOptions::default()).await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut payment_tx = mpt_payment(&issuer, &holder, "100", &issuance_id);
        test_transaction(&mut payment_tx, &issuer).await;

        let mut clawback_tx = mpt_clawback(&issuer, &holder, "1", &issuance_id);
        assert_transaction_engine_result(&mut clawback_tx, &issuer, "tecNO_PERMISSION").await;
    })
    .await;
}

#[tokio::test]
async fn test_mpt_clawback_reduces_holder_balance() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let issuance_id = create_issuance(
            &issuer,
            IssuanceOptions::default().with_flags([MPTokenIssuanceCreateFlag::TfMPTCanClawback]),
        )
        .await;

        let mut auth_tx = MPTokenAuthorize {
            common_fields: CommonFields {
                account: holder.classic_address.clone().into(),
                transaction_type: TransactionType::MPTokenAuthorize,
                ..Default::default()
            },
            mptoken_issuance_id: issuance_id.clone().into(),
            holder: None,
        };
        test_transaction(&mut auth_tx, &holder).await;

        let mut payment_tx = mpt_payment(&issuer, &holder, "9223372036854775807", &issuance_id);
        test_transaction(&mut payment_tx, &issuer).await;

        let node = mptoken_ledger_entry(&issuance_id, &holder.classic_address).await;
        assert_eq!(node["MPTAmount"], "9223372036854775807");

        let mut clawback_tx = mpt_clawback(&issuer, &holder, "500", &issuance_id);
        test_transaction(&mut clawback_tx, &issuer).await;

        let node = mptoken_ledger_entry(&issuance_id, &holder.classic_address).await;
        assert_eq!(node["MPTAmount"], "9223372036854775307");
    })
    .await;
}
