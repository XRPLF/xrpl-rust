use bigdecimal::Zero;
use std::str::FromStr;

use crate::common::{
    create_transferable_clawbackable_mptoken_issuance, generate_funded_wallet, get_client,
    lending_protocol::{
        account_objects_json, get_loan_broker_cover_available, get_loan_metadata, get_object_id,
        test_lending_transaction,
    },
    test_transaction,
    vault::get_vault_id,
    with_blockchain_lock,
};
use xrpl::{
    asynch::transaction::{autofill, sign, sign_and_submit},
    models::{
        requests::account_objects::AccountObjectType,
        transactions::{
            account_set::{AccountSet, AccountSetFlag},
            loan_broker_cover_clawback::LoanBrokerCoverClawback,
            loan_broker_cover_deposit::LoanBrokerCoverDeposit,
            loan_broker_cover_withdraw::LoanBrokerCoverWithdraw,
            loan_broker_delete::LoanBrokerDelete,
            loan_broker_set::LoanBrokerSet,
            loan_delete::LoanDelete,
            loan_manage::{LoanManage, LoanManageFlag},
            loan_pay::LoanPay,
            loan_set::LoanSet,
            mptoken_authorize::MPTokenAuthorize,
            payment::Payment,
            signer_list_set::{SignerEntry, SignerListSet},
            trust_set::TrustSet,
            vault_create::VaultCreate,
            vault_deposit::VaultDeposit,
            CommonFields, Transaction, TransactionType,
        },
        Amount, Currency, FlagCollection, IssuedCurrency, IssuedCurrencyAmount, MPTAmount,
        MPTCurrency, XRPAmount, XRP,
    },
    signing::sign_loan_set_by_counterparty,
    wallet::Wallet,
};

#[tokio::test]
async fn test_lending_protocol_lifecycle() {
    with_blockchain_lock(|| async {
        // The Vault Owner and Loan Broker must be on the same account
        let loan_issuer = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;
        let borrower_wallet = generate_funded_wallet().await;

        let vault_id = create_vault(
            &loan_issuer,
            Currency::XRP(XRP::new()),
            Some("1000"),
            Some(1),
        )
        .await;

        // Create a loan broker to capture attributes of the Lending Protocol
        let loan_broker_id = create_loan_broker(&loan_issuer, &vault_id, None).await;

        // Depositor deposits 100 into the vault
        //
        deposit_into_vault(
            &depositor_wallet,
            &vault_id,
            Amount::XRPAmount(XRPAmount("100".into())),
        )
        .await;

        // The Loan Broker and Borrower create a Loan object with a LoanSet
        // transaction and the requested principal (excluding fees) is transferred to
        // the Borrower.
        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
            None,
            Some(borrower_wallet.classic_address.as_str().into()),
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
            "100".into(),
            None,
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(1))
            .await
            .expect("Failed to auto-fill loan set transaction");

        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &borrower_wallet, false).unwrap();

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        // Loan cannot be deleted until all the remaining payments are completed
        assert_eq!(
            try_delete_loan(&loan_issuer, &loan_metadata.loan_id).await,
            "tecHAS_OBLIGATIONS"
        );

        impair_loan(&loan_issuer, &loan_metadata.loan_id).await;

        pay_loan(
            &borrower_wallet,
            &loan_metadata.loan_id,
            Amount::XRPAmount(XRPAmount("100".into())),
        )
        .await;
    })
    .await;
}

#[tokio::test]
async fn test_lending_protocol_with_mpt_and_multisigning() {
    with_blockchain_lock(|| async {
        // The Vault Owner and Loan Broker must be on the same account
        let loan_issuer = generate_funded_wallet().await;
        let mpt_issuer_wallet = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;
        let borrower_wallet = generate_funded_wallet().await;
        let signer1 = generate_funded_wallet().await;
        let signer2 = generate_funded_wallet().await;

        // Setup Multi-Signing
        setup_multisigning(&borrower_wallet, &signer1, &signer2).await;

        // Create Vault
        let vault_object = create_single_asset_vault(&loan_issuer, &mpt_issuer_wallet).await;

        // Depositor authorizes to hold MPT, then is funded by the issuer
        authorize_mpt(&depositor_wallet, &vault_object.mpt_issuance_id).await;

        send_mpt(
            &mpt_issuer_wallet,
            &depositor_wallet,
            "500000",
            &vault_object.mpt_issuance_id,
        )
        .await;

        // Loan Broker authorizes to hold MPT, then is funded by the issuer
        authorize_mpt(&loan_issuer, &vault_object.mpt_issuance_id).await;
        send_mpt(
            &mpt_issuer_wallet,
            &loan_issuer,
            "500000",
            &vault_object.mpt_issuance_id,
        )
        .await;

        deposit_into_vault(
            &depositor_wallet,
            &vault_object.vault_id,
            Amount::MPTAmount(MPTAmount {
                value: "200000".into(),
                mpt_issuance_id: vault_object.mpt_issuance_id.clone().into(),
            }),
        )
        .await;

        // Create LoanBroker ledger object to capture attributes of the Lending Protocol.
        // LoanBroker.DebtMaximum is 100000 and is capping the total amount the protocol
        // is owed (Defensive/Conservative lending)
        let loan_broker_id =
            create_loan_broker(&loan_issuer, &vault_object.vault_id, Some("150000")).await;

        // Create a Loan object.
        // The Loan Issuer creates the transaction from their account setting the
        // pre-agreed terms.
        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
            None,
            Some(borrower_wallet.classic_address.as_str().into()),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(0),
            None,
            None,
            None,
            "100000".into(),
            Some(1),
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(1)).await.unwrap();

        // Loan broker signs the transaction and sends it to the borrower
        // The Loan Issuer signs the transaction setting the SigningPubKey, TxnSignature, Signers, Account, Fee, Sequence fields.
        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        // Fails as loan borrower has not signed yet.
        test_lending_transaction(&mut loan_set_tx, "temBAD_SIGNER").await;

        assert!(
            loan_set_tx.get_common_fields().txn_signature.is_some(),
            "Transaction is missing a signature"
        );
        assert_eq!(
            loan_set_tx.get_common_fields().signing_pub_key,
            Some(loan_issuer.public_key.as_str().into()),
            "Transaction is missing a public key"
        );

        // Borrower signs the transaction and fills in the CounterpartySignature to confirm the
        // loan terms.
        sign_loan_set_by_counterparty(&mut loan_set_tx, &signer1, true).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &signer2, true).unwrap();

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        assert_eq!(loan_metadata.principal, loan_set_tx.principal_requested);
        assert_eq!(
            loan_metadata.borrower_address,
            borrower_wallet.classic_address
        );
        assert_eq!(
            Some(loan_metadata.payment_remaining),
            loan_set_tx.payment_total
        );

        // Test LoanBrokerCoverDeposit
        deposit_broker_cover(
            &loan_issuer,
            &loan_broker_id,
            Amount::MPTAmount(MPTAmount {
                value: "50000".into(),
                mpt_issuance_id: vault_object.mpt_issuance_id.clone().into(),
            }),
        )
        .await;

        // Assert LoanBroker object has updated CoverAvailable
        let deposit_cover_available =
            get_loan_broker_cover_available(&loan_issuer.classic_address).await;

        assert_eq!(deposit_cover_available, "50000");

        // Test LoanBrokerCoverWithdraw
        let withdraw_amount = withdraw_broker_cover(
            &loan_issuer,
            &loan_broker_id,
            Amount::MPTAmount(MPTAmount {
                value: "25000".into(),
                mpt_issuance_id: vault_object.mpt_issuance_id.clone().into(),
            }),
        )
        .await;

        // Assert LoanBroker object has updated CoverAvailable
        let withdrawcover_available = bigdecimal::BigDecimal::from_str(
            get_loan_broker_cover_available(&loan_issuer.classic_address)
                .await
                .as_str(),
        )
        .expect("Failed to parse cover available into bigdecimal");

        let diff = bigdecimal::BigDecimal::from_str(deposit_cover_available.as_str())
            .expect("Failed to parse deposit cover available into bigdecimal")
            - TryInto::<bigdecimal::BigDecimal>::try_into(withdraw_amount)
                .expect("Failed to parse loan broker cover withdraw amount into bigdecimal");

        assert_eq!(withdrawcover_available, diff);

        impair_loan(&loan_issuer, &loan_metadata.loan_id).await;

        // Assert Loan object is impaired
        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        assert_eq!(
            Some(loan_metadata.flags),
            Some(FlagCollection::new(vec![LoanManageFlag::TfLoanImpair]))
        );

        // Test LoanPay
        pay_loan(
            &borrower_wallet,
            &loan_metadata.loan_id,
            Amount::MPTAmount(MPTAmount {
                value: "100000".into(),
                mpt_issuance_id: vault_object.mpt_issuance_id.clone().into(),
            }),
        )
        .await;

        let loan_metadata = get_loan_metadata(
            borrower_wallet.classic_address.as_str(),
            AccountObjectType::Loan,
        )
        .await;

        // Loan gets un-impaired when a payment is made
        assert_eq!(
            Some(loan_metadata.flags.clone()),
            Some(FlagCollection::default())
        );
        // Entire loan is paid off
        assert!(
            bigdecimal::BigDecimal::from_str(loan_metadata.principal.as_str())
                .expect("Failed to parse loan metadata principal into bigdecimal")
                .is_zero(),
            "Principal should be zero"
        );

        // Test LoanDelete
        delete_loan(&loan_issuer, &loan_metadata.loan_id).await;

        let loan_delete_result =
            account_objects_json(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        assert_eq!(
            loan_delete_result["account_objects"]
                .as_array()
                .expect("account_objects array missing")
                .len(),
            0,
            "Loan should be deleted"
        );

        // Test LoanBrokerCoverClawback
        clawback_broker_cover(
            &mpt_issuer_wallet,
            &loan_broker_id,
            Amount::MPTAmount(MPTAmount {
                value: "10000".into(),
                mpt_issuance_id: vault_object.mpt_issuance_id.clone().into(),
            }),
        )
        .await;

        let loan_broker_cover_clawback_result =
            account_objects_json(&loan_issuer.classic_address, AccountObjectType::LoanBroker).await;

        let cover_available: bigdecimal::BigDecimal = loan_broker_cover_clawback_result
            ["account_objects"]
            .as_array()
            .expect("account_objects array missing")[0]["CoverAvailable"]
            .as_str()
            .expect("CoverAvailable missing")
            .parse()
            .expect("Failed to parse CoverAvailable");

        let diff = withdrawcover_available - bigdecimal::BigDecimal::from(10000);

        assert_eq!(cover_available, diff);

        // Test LoanBrokerDelete
        delete_loan_broker(&loan_issuer, &loan_broker_id).await;

        let loan_broker_delete_result =
            account_objects_json(&loan_issuer.classic_address, AccountObjectType::LoanBroker).await;

        assert_eq!(
            loan_broker_delete_result["account_objects"]
                .as_array()
                .expect("account_objects array missing")
                .len(),
            0,
            "Loan Broker should be deleted"
        );
    })
    .await;
}

#[tokio::test]
async fn test_loan_set_txn_counterparty_is_loan_broker_owner() {
    with_blockchain_lock(|| async {
        let loan_issuer = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;

        let vault_id = create_vault(
            &loan_issuer,
            Currency::XRP(XRP::new()),
            Some("1000"),
            Some(1),
        )
        .await;

        let loan_broker_id = create_loan_broker(&loan_issuer, &vault_id, None).await;

        // Depositor deposits 100 into the vault
        deposit_into_vault(
            &depositor_wallet,
            &vault_id,
            Amount::XRPAmount(XRPAmount("100".into())),
        )
        .await;

        //  The Loan Broker and Borrower (Borrower is the Owner of the
        // LoanBroker, i.e. loan_issuer account) create a Loan object with a LoanSet
        // transaction and the requested principal (excluding fees) is transferred to
        // the Borrower.
        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
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
            None,
            None,
            "100".into(),
            None,
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(0))
            .await
            .expect("Failed to auto-fill loan set transaction");

        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &loan_issuer, false).unwrap();

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&loan_issuer.classic_address, AccountObjectType::Loan).await;

        assert!(!loan_metadata.is_repaid);
    })
    .await
}

#[tokio::test]
async fn test_lending_protocol_lifecycle_with_iou_asset() {
    with_blockchain_lock(|| async {
        let loan_issuer = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;
        let borrower_wallet = generate_funded_wallet().await;

        // Set up the relevant flags on the loan_issuer account -- This is
        // a pre-requisite for a Vault to hold the Issued Currency Asset
        let mut account_set = AccountSet::new(
            loan_issuer.classic_address.as_str().into(),
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
            None,
            None,
            None,
            Some(AccountSetFlag::AsfDefaultRipple),
            None,
            None,
            None,
        );

        test_transaction(&mut account_set, &loan_issuer).await;

        let mut trust_set = TrustSet::new(
            depositor_wallet.classic_address.as_str().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: loan_issuer.classic_address.as_str().into(),
                value: "1000".into(),
            },
            None,
            None,
        );

        test_transaction(&mut trust_set, &depositor_wallet).await;

        let mut trust_set = TrustSet::new(
            borrower_wallet.classic_address.as_str().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: loan_issuer.classic_address.as_str().into(),
                value: "1000".into(),
            },
            None,
            None,
        );

        test_transaction(&mut trust_set, &borrower_wallet).await;

        // Transfer some IOUs from the issuer to LoanBroker
        let mut payment_tx = Payment::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: loan_issuer.classic_address.clone().into(),
                value: "1000".into(),
            }),
            depositor_wallet.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
        );

        test_transaction(&mut payment_tx, &loan_issuer).await;

        let vault_id = create_vault(
            &loan_issuer,
            Currency::IssuedCurrency(IssuedCurrency::new(
                "USD".into(),
                loan_issuer.classic_address.clone().into(),
            )),
            Some("10000"),
            Some(1),
        )
        .await;

        let loan_broker_id = create_loan_broker(&loan_issuer, &vault_id, Some("10000")).await;

        // Depositor deposits 1000 into the vault
        deposit_into_vault(
            &depositor_wallet,
            &vault_id,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: loan_issuer.classic_address.clone().into(),
                value: "1000".into(),
            }),
        )
        .await;

        //  The Loan Broker and Borrower create a Loan object with a LoanSet
        // transaction and the requested principal (excluding fees) is transferred to
        // the Borrower.
        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
            None,
            Some(borrower_wallet.classic_address.as_str().into()),
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
            "100".into(),
            None,
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(1))
            .await
            .expect("Failed to auto-fill loan set transaction");

        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &borrower_wallet, false).unwrap();

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        // Delete the Loan object
        // Test LoanDelete
        // Loan cannot be deleted until all the remaining payments are completed
        assert_eq!(
            try_delete_loan(&loan_issuer, &loan_metadata.loan_id).await,
            "tecHAS_OBLIGATIONS"
        );

        impair_loan(&loan_issuer, &loan_metadata.loan_id).await;

        pay_loan(
            &borrower_wallet,
            &loan_metadata.loan_id,
            Amount::IssuedCurrencyAmount(IssuedCurrencyAmount {
                currency: "USD".into(),
                issuer: loan_issuer.classic_address.clone().into(),
                value: "100".into(),
            }),
        )
        .await;
    })
    .await
}

#[tokio::test]
async fn test_loan_set_with_sign_loan_set_by_counterparty() {
    with_blockchain_lock(|| async {
        let loan_issuer = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;
        let borrower_wallet = generate_funded_wallet().await;

        let vault_id = create_vault(
            &loan_issuer,
            Currency::XRP(XRP::new()),
            Some("1000"),
            Some(1),
        )
        .await;
        let loan_broker_id = create_loan_broker(&loan_issuer, &vault_id, Some("10000")).await;

        // Depositor deposits 100 into the vault
        deposit_into_vault(
            &depositor_wallet,
            &vault_id,
            Amount::XRPAmount(XRPAmount("100".into())),
        )
        .await;

        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
            None,
            Some(borrower_wallet.classic_address.as_str().into()),
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
            "100".into(),
            None,
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(1))
            .await
            .expect("Failed to auto-fill loan set transaction");

        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &borrower_wallet, false).unwrap();

        assert!(loan_set_tx.counterparty_signature.is_some());
        assert!(loan_set_tx
            .counterparty_signature
            .as_ref()
            .expect("Missing CounterpartySignature")
            .signing_pub_key
            .is_some());
        assert!(loan_set_tx
            .counterparty_signature
            .as_ref()
            .expect("Missing CounterpartySignature")
            .txn_signature
            .is_some());

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        assert!(!loan_metadata.is_repaid);
    })
    .await;
}

#[tokio::test]
async fn test_loan_set_with_combine_loanset_counterparty_signers() {
    with_blockchain_lock(|| async {
        // The Vault Owner and Loan Broker must be on the same account
        let loan_issuer = generate_funded_wallet().await;
        let depositor_wallet = generate_funded_wallet().await;
        let borrower_wallet = generate_funded_wallet().await;
        let signer1 = generate_funded_wallet().await;
        let signer2 = generate_funded_wallet().await;

        // Setup Multi-Signing
        setup_multisigning(&borrower_wallet, &signer1, &signer2).await;

        let vault_id = create_vault(
            &loan_issuer,
            Currency::XRP(XRP::new()),
            Some("1000"),
            Some(1),
        )
        .await;
        let loan_broker_id = create_loan_broker(&loan_issuer, &vault_id, Some("10000")).await;

        // Depositor deposits 100 into the vault
        deposit_into_vault(
            &depositor_wallet,
            &vault_id,
            Amount::XRPAmount(XRPAmount("100".into())),
        )
        .await;

        let mut loan_set_tx = LoanSet::new(
            loan_issuer.classic_address.clone().into(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            loan_broker_id.clone().into(),
            None,
            Some(borrower_wallet.classic_address.as_str().into()),
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
            "100".into(),
            None,
            None,
            None,
        );

        let client = get_client().await;

        autofill(&mut loan_set_tx, client, Some(1))
            .await
            .expect("Failed to auto-fill loan set transaction");

        sign(&mut loan_set_tx, &loan_issuer, false).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &signer1, true).unwrap();

        sign_loan_set_by_counterparty(&mut loan_set_tx, &signer2, true).unwrap();

        assert!(loan_set_tx.counterparty_signature.is_some());
        assert!(loan_set_tx
            .counterparty_signature
            .as_ref()
            .unwrap()
            .signers
            .is_some());
        assert_eq!(
            loan_set_tx
                .counterparty_signature
                .as_ref()
                .unwrap()
                .signers
                .as_ref()
                .unwrap()
                .len(),
            2
        );

        test_lending_transaction(&mut loan_set_tx, "tesSUCCESS").await;

        let loan_metadata =
            get_loan_metadata(&borrower_wallet.classic_address, AccountObjectType::Loan).await;

        assert!(!loan_metadata.is_repaid);
    })
    .await;
}

/// Creates a Vault with the given `currency` and returns its vault id.
async fn create_vault<'a>(
    loan_issuer: &Wallet,
    currency: Currency<'a>,
    assets_maximum: Option<&str>,
    withdrawal_policy: Option<u8>,
) -> String {
    let mut vault_create_tx = VaultCreate::new(
        loan_issuer.classic_address.as_str().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        currency,
        None,
        assets_maximum.map(|v| v.into()),
        None,
        None,
        withdrawal_policy,
        None,
    );

    let client = get_client().await;

    sign_and_submit(&mut vault_create_tx, client, loan_issuer, true, true)
        .await
        .expect("create_vault: sign_and_submit failed");

    get_vault_id(&loan_issuer.classic_address).await
}

/// Creates a single-asset (MPT-backed) vault, issuing a fresh transferable,
/// clawbackable MPT for it first.
async fn create_single_asset_vault(
    loan_issuer: &Wallet,
    mpt_issuer_wallet: &Wallet,
) -> VaultObject {
    let mpt_issuance_id =
        create_transferable_clawbackable_mptoken_issuance(mpt_issuer_wallet).await;

    let vault_id = create_vault(
        loan_issuer,
        Currency::MPTCurrency(MPTCurrency::new(mpt_issuance_id.clone().into())),
        None,
        None,
    )
    .await;

    VaultObject {
        mpt_issuance_id,
        vault_id,
    }
}

/// Creates a LoanBroker ledger object over `vault_id` and returns its object id.
async fn create_loan_broker(
    loan_issuer: &Wallet,
    vault_id: &str,
    debt_maximum: Option<&str>,
) -> String {
    let mut loan_broker_set_tx = LoanBrokerSet::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        vault_id.into(),
        None,
        None,
        debt_maximum.map(|v| v.into()),
        None,
        None,
    );

    test_transaction(&mut loan_broker_set_tx, loan_issuer).await;

    get_object_id(&loan_issuer.classic_address, AccountObjectType::LoanBroker).await
}

/// Deposits `amount` into the vault on behalf of `depositor`.
async fn deposit_into_vault<'a>(depositor: &Wallet, vault_id: &str, amount: Amount<'a>) {
    let mut vault_deposit_tx = VaultDeposit::new(
        depositor.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        vault_id.into(),
        amount,
    );

    test_transaction(&mut vault_deposit_tx, depositor).await;
}

/// Impairs the loan identified by `loan_id`.
async fn impair_loan(loan_issuer: &Wallet, loan_id: &str) {
    let mut loan_manage_tx = LoanManage::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        Some(FlagCollection::new(vec![LoanManageFlag::TfLoanImpair])),
        None,
        None,
        None,
        None,
        None,
        None,
        loan_id.into(),
    );

    test_transaction(&mut loan_manage_tx, loan_issuer).await;
}

/// Makes a payment of `amount` towards the loan identified by `loan_id`.
async fn pay_loan<'a>(borrower: &Wallet, loan_id: &str, amount: Amount<'a>) {
    let mut loan_pay_tx = LoanPay::new(
        borrower.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_id.into(),
        amount,
    );

    test_transaction(&mut loan_pay_tx, borrower).await;
}

/// Attempts to delete the loan and returns the engine result, so callers can
/// assert on a rejection such as `tecHAS_OBLIGATIONS`.
async fn try_delete_loan(loan_issuer: &Wallet, loan_id: &str) -> String {
    let mut loan_delete_tx = LoanDelete::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_id.into(),
    );

    let client = get_client().await;
    let response = sign_and_submit(&mut loan_delete_tx, client, loan_issuer, true, true)
        .await
        .unwrap();

    response.engine_result.into()
}

/// Deletes the loan, asserting on the normal `test_transaction` success path.
async fn delete_loan(loan_issuer: &Wallet, loan_id: &str) {
    let mut loan_delete_tx = LoanDelete::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_id.into(),
    );

    test_transaction(&mut loan_delete_tx, loan_issuer).await;
}

/// Deletes the loan broker identified by `loan_broker_id`.
async fn delete_loan_broker(loan_issuer: &Wallet, loan_broker_id: &str) {
    let mut loan_broker_delete_tx = LoanBrokerDelete::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_broker_id.into(),
    );

    test_transaction(&mut loan_broker_delete_tx, loan_issuer).await;
}

/// Deposits `amount` as cover for the loan broker identified by `loan_broker_id`.
async fn deposit_broker_cover<'a>(loan_issuer: &Wallet, loan_broker_id: &str, amount: Amount<'a>) {
    let mut tx = LoanBrokerCoverDeposit::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_broker_id.into(),
        amount,
    );

    test_transaction(&mut tx, loan_issuer).await;
}

/// Withdraws `amount` of cover from the loan broker identified by
/// `loan_broker_id`, returning the withdrawn `Amount` for callers that need
/// it for follow-up assertions.
async fn withdraw_broker_cover<'a>(
    loan_issuer: &Wallet,
    loan_broker_id: &'a str,
    amount: Amount<'a>,
) -> Amount<'a> {
    let mut tx = LoanBrokerCoverWithdraw::new(
        loan_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        loan_broker_id.into(),
        amount,
        None,
        None,
    );

    test_transaction(&mut tx, loan_issuer).await;

    tx.amount
}

/// Claws back `amount` of cover from the loan broker identified by
/// `loan_broker_id`. Called by the MPT issuer.
async fn clawback_broker_cover<'a>(
    mpt_issuer: &Wallet,
    loan_broker_id: &'a str,
    amount: Amount<'a>,
) {
    let mut tx = LoanBrokerCoverClawback::new(
        mpt_issuer.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(loan_broker_id.into()),
        Some(amount),
    );

    test_transaction(&mut tx, mpt_issuer).await;
}

/// Opts `wallet` in to holding the MPT identified by `mpt_issuance_id`.
async fn authorize_mpt(wallet: &Wallet, mpt_issuance_id: &str) {
    let mut tx = MPTokenAuthorize {
        common_fields: CommonFields {
            account: wallet.classic_address.clone().into(),
            transaction_type: TransactionType::MPTokenAuthorize,
            ..Default::default()
        },
        mptoken_issuance_id: mpt_issuance_id.into(),
        holder: None, // omitted when a holder opts in themselves
    };

    test_transaction(&mut tx, wallet).await;
}

/// Sends `value` of the MPT identified by `mpt_issuance_id` from `from` to `to`.
async fn send_mpt(from: &Wallet, to: &Wallet, value: &str, mpt_issuance_id: &str) {
    let value = value.to_string();
    let mpt_issuance_id = mpt_issuance_id.to_string();

    let mut tx = Payment::new(
        from.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Amount::MPTAmount(MPTAmount {
            value: value.into(),
            mpt_issuance_id: mpt_issuance_id.into(),
        }),
        to.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
    );

    test_transaction(&mut tx, from).await;
}

struct VaultObject {
    pub mpt_issuance_id: String,
    pub vault_id: String,
}

pub async fn setup_multisigning(wallet: &Wallet, signer1: &Wallet, signer2: &Wallet) {
    let mut transaction = SignerListSet::new(
        wallet.classic_address.clone().into(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        2,
        Some(vec![
            SignerEntry::new(signer1.classic_address.clone(), 1),
            SignerEntry::new(signer2.classic_address.clone(), 1),
        ]),
    );

    test_transaction(&mut transaction, wallet).await;
}
