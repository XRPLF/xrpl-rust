use serde_json::Value;

#[cfg(feature = "std")]
use strum::IntoEnumIterator;
use xrpl::models::{
    requests::{
        account_objects::{AccountObjectType, AccountObjects},
        CommonFields as ReqCommonFields, RequestMethod,
    },
    transactions::loan_manage::LoanManageFlag,
    FlagCollection,
};

use super::{get_ledger_close_time, ledger_accept, wait_for_ledger_close_time};

#[cfg(feature = "std")]
use xrpl::asynch::clients::XRPLAsyncClient;

/// Build an `AccountObjects` request that filters for `AccountObjectType` entries owned by `owner`.
pub fn ao_request(owner: &str, ao: AccountObjectType) -> AccountObjects<'_> {
    AccountObjects {
        common_fields: ReqCommonFields {
            command: RequestMethod::AccountObjects,
            id: None,
        },
        account: owner.into(),
        ledger_lookup: None,
        r#type: Some(ao),
        deletion_blockers_only: None,
        limit: None,
        marker: None,
    }
}

/// Fetch all object `account_objects` for `owner` as raw JSON.
#[cfg(feature = "std")]
pub async fn account_objects_json(owner: &str, ao: AccountObjectType) -> Value {
    let client = super::get_client().await;
    let resp = client
        .request(ao_request(owner, ao).into())
        .await
        .expect("account_objects request failed");

    resp.raw_result.expect(
        "account_objects response contained no raw_result — server may have returned an error",
    )
}

/// Fetch the metadata for the first loan  `account_objects` for `owner`.
///
///
///
/// Active Loan Object
/// Object {
/// "account": String("rstX4kh7hrrVu1AmXaSU3pkMnfs1vHRGp6"),
/// "account_objects": Array [
///     Object {
///         "Borrower": String("rstX4kh7hrrVu1AmXaSU3pkMnfs1vHRGp6"),
///         "Flags": Number(0),
///         "GracePeriod": Number(60),
///         "LedgerEntryType": String("Loan"),
///         "LoanBrokerID": String("AA56AAE05414CE40DAD74325D4F40E536E09B43A106F4508FFB5DA4B31527BDC"),
///         "LoanBrokerNode": String("0"),
///         "LoanSequence": Number(1),
///         "NextPaymentDueDate": Number(839600564),
///         "OwnerNode": String("0"),
///         "PaymentInterval": Number(60),
///         "PaymentRemaining": Number(1),
///         "PeriodicPayment": String("50000"),
///         "PreviousPaymentDueDate": Number(839600504),
///         "PreviousTxnID": String("24AF3F8F6D38E5EF9E7B22B8A3CEB428FF79B11025D179106EB678DC4814A4AE"),
///         "PreviousTxnLgrSeq": Number(1867),
///         "PrincipalOutstanding": String("50000"),
///         "StartDate": Number(839600444),
///         "TotalValueOutstanding": String("50000"),
///         "index": String("74FF4D1D03CF31A7E47C9E42F5BA9CF4A0644C57D074D3AA48D14B86B48E2B4F"),
///     },
/// ],
/// "ledger_current_index": Number(1868),
/// "status": String("success"),
/// "validated": Bool(false),
/// }
///
/// Fully Paid Loan Object
/// Object {
///     "account": String("radbF7vy7fxbSQimjs8KYvTHktUuMBDWSL"),
///     "account_objects": Array [
///         Object {
///             "Borrower": String("radbF7vy7fxbSQimjs8KYvTHktUuMBDWSL"),
///             "Flags": Number(0),
///             "GracePeriod": Number(60),
///             "LedgerEntryType": String("Loan"),
///             "LoanBrokerID": String("E792BBBC0F3D865E6AA5E9B2E5CA76EBC27D44DF12270815CB5CC7ABA906C28C"),
///             "LoanBrokerNode": String("0"),
///             "LoanSequence": Number(1),
///             "OwnerNode": String("0"),
///             "PaymentInterval": Number(60),
///             "PeriodicPayment": String("100000"),
///             "PreviousPaymentDueDate": Number(839601184),
///             "PreviousTxnID": String("57984B9C18C56C343E17C810C0DE7CD01B9DBACF726A4CB898BB9A33B0864D08"),
///             "PreviousTxnLgrSeq": Number(1885),
///             "StartDate": Number(839601124),
///             "index": String("0901B6E91B205A6F7F3567F9609C8B56880CE12EAB75A2F0C67C3DD3804EC4A0"),
///         },
///     ],
///     "ledger_current_index": Number(1886),
///     "status": String("success"),
///     "validated": Bool(false),
/// }
///
///
///
///
#[cfg(feature = "std")]
pub async fn get_loan_metadata(owner: &str, ao: AccountObjectType) -> LoanMetadata {
    let resp = account_objects_json(owner, ao.clone()).await;
    let objects = resp["account_objects"]
        .as_array()
        .expect("account_objects array missing");

    assert!(!objects.is_empty(), "no {ao} found for {owner}");

    let object = &objects[0];

    let loan_id = require_str(object, "index");

    let loan_broker_id = require_str(object, "LoanBrokerID");

    let borrower_address = require_str(object, "Borrower");

    let principal = optional_amount_str(object, "PrincipalOutstanding");

    let payment_remaining = optional_u32(object, "PaymentRemaining");
    let is_repaid = object.get("PrincipalOutstanding").is_none();

    let flags_num = object["Flags"]
        .as_u64()
        .expect("Flags missing or not a number") as u32;

    let flags: FlagCollection<LoanManageFlag> = FlagCollection::new(
        LoanManageFlag::iter()
            .filter(|flag| flags_num & (*flag as u32) != 0)
            .collect(),
    );

    LoanMetadata {
        loan_id,
        principal,
        loan_broker_id,
        borrower_address,
        payment_remaining,
        flags,
        is_repaid,
    }
}

#[derive(Debug)]
pub struct LoanMetadata {
    pub loan_id: String,
    pub principal: String,
    pub loan_broker_id: String,
    pub borrower_address: String,
    pub payment_remaining: u32,
    pub flags: FlagCollection<LoanManageFlag>,
    /// true once the loan has been fully repaid (PrincipalOutstanding /
    /// PaymentRemaining omitted from the ledger object)
    pub is_repaid: bool,
}

/// Fetch the metadata for the first loan  `account_objects` for `owner`.
#[cfg(feature = "std")]
pub async fn get_loan_broker_cover_available(owner: &str) -> String {
    let client = super::get_client().await;

    let resp = client
        .request(ao_request(owner, AccountObjectType::LoanBroker).into())
        .await
        .expect("account_objects request failed");

    let resp = resp.raw_result.expect(
        "ledger_entry response contained no raw_result — server may have returned an error",
    );

    resp["account_objects"][0]["CoverAvailable"]
        .as_str()
        .expect("CoverAvailable missing")
        .to_string()
}

/// Return the ledger object ID (`index`) of the first object owned by `owner`.
///
/// Panics if no vault is found.
#[cfg(feature = "std")]
pub async fn get_object_id(owner: &str, ao: AccountObjectType) -> String {
    let resp = account_objects_json(owner, ao.clone()).await;
    let objects = resp["account_objects"]
        .as_array()
        .expect("account_objects array missing");

    assert!(!objects.is_empty(), "no {ao} found for {owner}");

    objects[0]["index"]
        .as_str()
        .expect("object id missing")
        .to_string()
}

#[cfg(feature = "std")]
pub async fn test_lending_transaction<'a, T, F>(tx: &mut T, tx_result: &str)
where
    T: xrpl::models::transactions::Transaction<'a, F>
        + xrpl::models::Model
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + core::fmt::Debug,
    F: strum::IntoEnumIterator + serde::Serialize + core::fmt::Debug + PartialEq + Clone + 'a,
{
    use xrpl::asynch::transaction::submit;
    let client = super::get_client().await;

    // Record the validated ledger close_time before submission so we can
    // detect that a new ledger has been validated after the transaction lands.
    let pre_close = get_ledger_close_time().await;

    let result = submit(tx, client)
        .await
        .expect("test_transaction: submit failed");

    assert_eq!(
        result.engine_result, tx_result,
        "Expected {} but got: {} — {}",
        tx_result, result.engine_result, result.engine_result_message
    );

    // Advance the ledger and wait until a new validated ledger has closed,
    // ensuring the transaction is in validated state before returning.
    ledger_accept().await;
    wait_for_ledger_close_time(pre_close + 1).await;
}

/// Required string field — panics if missing, since these should always be present.
fn require_str(object: &serde_json::Value, field: &str) -> String {
    object[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} missing"))
        .to_string()
}

/// Optional string field that rippled omits once it defaults to zero
/// (e.g. PrincipalOutstanding / TotalValueOutstanding after full repayment).
fn optional_amount_str(object: &serde_json::Value, field: &str) -> String {
    object
        .get(field)
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string()
}

/// Optional numeric field that rippled omits once it defaults to zero
/// (e.g. PaymentRemaining after full repayment).
fn optional_u32(object: &serde_json::Value, field: &str) -> u32 {
    object.get(field).and_then(|v| v.as_u64()).unwrap_or(0) as u32
}
