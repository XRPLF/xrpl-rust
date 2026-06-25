// xrpl.org reference:
//   https://xrpl.org/docs/references/protocol/transactions/types/payment
//
// Scenarios:
//   - mpt_payment: MPTAmount Payment between two authorized accounts
//
// Validates that MPTAmount serializes/deserializes correctly through the wire
// format when submitted to a live rippled node.

use crate::common::{
    create_transferable_mptoken_issuance, generate_funded_wallet, get_client, test_transaction,
    with_blockchain_lock,
};
use xrpl::asynch::clients::XRPLAsyncClient;
use xrpl::models::{
    requests::account_objects::{AccountObjectType, AccountObjects},
    results,
    transactions::{
        mptoken_authorize::MPTokenAuthorize, payment::Payment, CommonFields, TransactionType,
    },
    Amount, MPTAmount,
};

#[tokio::test]
async fn test_mpt_payment() {
    with_blockchain_lock(|| async {
        let issuer = generate_funded_wallet().await;
        let holder = generate_funded_wallet().await;
        let client = get_client().await;

        // 1. Create MPT issuance with TfMPTCanTransfer
        let issuance_id = create_transferable_mptoken_issuance(&issuer).await;

        // 2. Holder opts in to hold the MPT
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

        // 3. Issuer pays 1000 MPT to holder
        let mut payment = Payment {
            common_fields: CommonFields {
                account: issuer.classic_address.clone().into(),
                transaction_type: TransactionType::Payment,
                ..Default::default()
            },
            amount: Amount::MPTAmount(MPTAmount {
                value: "1000".into(),
                mpt_issuance_id: issuance_id.clone().into(),
            }),
            destination: holder.classic_address.clone().into(),
            ..Default::default()
        };
        test_transaction(&mut payment, &issuer).await;

        // 4. Verify deserialization round-trip: query holder's MPToken object
        //    and assert the on-ledger MPTAmount field matches what was sent.
        let ao_response = client
            .request(
                AccountObjects::new(
                    None,
                    holder.classic_address.clone().into(),
                    None,
                    None,
                    Some(AccountObjectType::Mptoken),
                    None,
                    None,
                    None,
                )
                .into(),
            )
            .await
            .expect("account_objects request failed");

        let ao_result: results::account_objects::AccountObjects<'_> = ao_response
            .try_into()
            .expect("failed to parse account_objects");

        let mptoken = ao_result
            .account_objects
            .iter()
            .find(|obj| {
                obj.get("MPTokenIssuanceID")
                    .and_then(|v| v.as_str())
                    .map(|id| id.eq_ignore_ascii_case(&issuance_id))
                    .unwrap_or(false)
            })
            .expect("MPToken object not found for issuance");

        let on_ledger_amount = mptoken["MPTAmount"]
            .as_str()
            .expect("MPTAmount field missing or not a string");

        assert_eq!(
            on_ledger_amount, "1000",
            "MPTAmount deserialized from ledger ({on_ledger_amount}) \
             does not match the sent value (1000)"
        );
    })
    .await;
}
